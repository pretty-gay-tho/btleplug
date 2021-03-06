// btleplug Source Code File
//
// Copyright 2020 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.
//
// Some portions of this file are taken and/or modified from Rumble
// (https://github.com/mwylde/rumble), using a dual MIT/Apache License under the
// following copyright:
//
// Copyright (c) 2014 The Rust Project Developers

use crate::{
    api::{
        Central, CentralEvent, EventHandler, BDAddr
    },
    Result
};
use super::{
    peripheral::Peripheral,
    ble::watcher::BLEWatcher,
    utils,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Adapter {
    watcher: Arc<Mutex<BLEWatcher>>,
    peripherals: Arc<Mutex<HashMap<BDAddr, Peripheral>>>,
    event_handlers: Arc<Mutex<Vec<EventHandler>>>,
}

impl Adapter {
    pub fn new() -> Self {
        let watcher = Arc::new(Mutex::new(BLEWatcher::new()));
        let peripherals = Arc::new(Mutex::new(HashMap::new()));
        let event_handlers = Arc::new(Mutex::new(Vec::new()));
        Adapter { watcher, peripherals, event_handlers }
    }

    pub fn emit(&self, event: CentralEvent) {
        debug!("emitted {:?}", event);
        let handlers = self.event_handlers.clone();
        let vec = handlers.lock().unwrap();
        for handler in (*vec).iter() {
            handler(event.clone());
        }
    }
}

impl Central<Peripheral> for Adapter {
    fn on_event(&self, handler: EventHandler) {
        let list = self.event_handlers.clone();
        list.lock().unwrap().push(handler);
    }

    fn start_scan(&self) -> Result<()> {
        let peripherals = self.peripherals.clone();
        let watcher = self.watcher.lock().unwrap();
        let adapter = self.clone();
        watcher.start(Box::new(move |args| {
            let bluetooth_address = args.get_bluetooth_address().unwrap();
            let address = utils::to_addr(bluetooth_address);
            let mut peripherals = peripherals.lock().unwrap();
            let peripheral = peripherals.entry(address).or_insert_with(|| {
                Peripheral::new(adapter.clone(), address)
            });
            peripheral.update_properties(&args);
            adapter.emit(CentralEvent::DeviceDiscovered(address));
        }))
    }

    fn stop_scan(&self) -> Result<()> {
        let watcher = self.watcher.lock().unwrap();
        watcher.stop().unwrap();
        Ok(())
    }

    fn peripherals(&self) -> Vec<Peripheral> {
        let l = self.peripherals.lock().unwrap();
        l.values().cloned().collect()
    }

    fn peripheral(&self, address: BDAddr) -> Option<Peripheral> {
        let l = self.peripherals.lock().unwrap();
        l.get(&address).cloned()
    }

    fn active(&self, enabled: bool) {
    }

    fn filter_duplicates(&self, enabled: bool) {
    }
}
