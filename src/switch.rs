use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use uuid::Uuid;

const PLUG_CHARACTERISTIC_TO_DEVICE_UUID: Uuid =
    uuid::uuid!("cba20002-224d-11e6-9fb8-0002a5d5c51b");
// const PLUG_CHARACTERISTIC_TO_TERM_UUID: Uuid = uuid::uuid!("cba20003-224d-11e6-9fb8-0002a5d5c51b");

const PLUG_ON_CMD: [u8; 6] = [0x57, 0xf, 0x50, 0x1, 0x1, 0x80];
const PLUG_OFF_CMD: [u8; 6] = [0x57, 0xf, 0x50, 0x1, 0x1, 0x0];
// const PLUG_READ_STATE_CMD: [u8;4] = [0x57, 0xf, 0x51, 0x1];

const PLUG_ON_STATE: u8 = 0x80;
const PLUG_OFF_STATE: u8 = 0x0;

const MANUFACTURE_DATA_SWITCH_STATE_POS: usize = 7;
const WAIT_FOR_SCAN: u64 = 2;

#[derive(Debug, PartialEq)]
pub enum SwitchState {
    ON,
    OFF,
}

#[derive(Debug)]
pub struct Switch {
    target_mac: String,
    adapter: Option<Adapter>,
    peripheral: Option<Peripheral>,
    write_dev_characteristic: Option<Characteristic>,
    is_debug: bool,
}

impl Switch {
    pub async fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        log::debug!("try BLE find manager");
        let manager = Manager::new().await.unwrap();

        log::debug!("try BLE find adapters");
        let adapters = manager.adapters().await.unwrap();

        log::debug!("try BLE find central");
        let adapter = adapters.into_iter().nth(0).unwrap();

        log::debug!("try BLE start scanning");
        adapter.start_scan(ScanFilter::default()).await.unwrap();
        log::debug!("start BLE scanning");

        sleep(Duration::from_secs(WAIT_FOR_SCAN));

        let peripheral = self.find_switch_plug(&adapter).await.unwrap();

        // connect to the device
        log::debug!("connecting to device");
        match peripheral.connect().await {
            Err(e) => {
                return Err(Box::new(e));
            }
            _ => {}
        }

        log::debug!("connected to device");

        // discover services and characteristics
        log::debug!("try start discover_services");
        peripheral.discover_services().await.unwrap();
        log::debug!("discover_services started");

        // find the characteristic we want
        log::debug!("try find device write characteristic");
        let characteristics = peripheral.characteristics();
        let write_char = characteristics
            .iter()
            .find(|c| c.uuid == PLUG_CHARACTERISTIC_TO_DEVICE_UUID)
            .unwrap()
            .clone();

        self.write_dev_characteristic = Some(write_char);
        log::debug!("found device write characteristic");

        self.peripheral = Some(peripheral);
        self.adapter = Some(adapter);

        log::info!("device setup succeed");
        Ok(())
    }

    pub async fn send_on_cmd(&self) {
        log::info!("trigger target device on");
        self.peripheral
            .as_ref()
            .unwrap()
            .write(
                self.write_dev_characteristic.as_ref().unwrap(),
                &PLUG_ON_CMD,
                WriteType::WithoutResponse,
            )
            .await
            .unwrap();
    }

    pub async fn send_off_cmd(&self) {
        log::info!("trigger target device off");
        self.peripheral
            .as_ref()
            .unwrap()
            .write(
                self.write_dev_characteristic.as_ref().unwrap(),
                &PLUG_OFF_CMD,
                WriteType::WithoutResponse,
            )
            .await
            .unwrap();
    }

    pub async fn get_current_state(&self) -> Option<SwitchState> {
        let props = self
            .peripheral
            .as_ref()
            .unwrap()
            .properties()
            .await
            .unwrap()
            .unwrap();

        if self.is_debug {
            for s in props.services.iter() {
                log::debug!("service uuid: {s:?}");
            }

            for (k, s_data) in props.service_data.iter() {
                log::debug!("service key {k:?} data: {s_data:?}");
            }

            for (k, m_data) in props.manufacturer_data.iter() {
                log::debug!("manufacturer key {k:?} data: {m_data:?}");

                for m_d in m_data {
                    log::debug!("data in binary: {m_d:#010b} - {m_d:#010x}");
                }
            }
        }

        for (_, m_data) in props.manufacturer_data {
            if m_data[MANUFACTURE_DATA_SWITCH_STATE_POS] == PLUG_ON_STATE {
                return Some(SwitchState::ON);
            }

            if m_data[MANUFACTURE_DATA_SWITCH_STATE_POS] == PLUG_OFF_STATE {
                return Some(SwitchState::OFF);
            }
        }

        None
    }

    async fn find_switch_plug(&self, adapter: &Adapter) -> Option<Peripheral> {
        for p in adapter.peripherals().await.unwrap() {
            if p.address().to_string() == self.target_mac {
                log::debug!("found target device {p:?}");
                log::info!("found target device");
                return Some(p);
            }
        }

        None
    }

    pub async fn disconnect(&mut self) {
        if let Some(p) = &self.peripheral {
            match p.disconnect().await {
                Err(e) => {
                    log::warn!("error on disconnection {e:?}");
                }
                _ => {}
            }
        }

        if let Some(a) = &self.adapter {
            match a.stop_scan().await {
                Err(e) => {
                    log::warn!("error on stop scanning {e:?}");
                }
                _ => {}
            };
        }
        log::info!("disconnected from BLE device");
    }
}

pub fn new(target_mac: &str, is_debug: bool) -> Switch {
    Switch {
        target_mac: target_mac.to_string(),
        adapter: None,
        peripheral: None,
        write_dev_characteristic: None,
        is_debug,
    }
}
