use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

pub type EnvironmentData = dht22_pi::Reading;

const READ_WAIT_SEC: u64 = 3;

pub fn start_read(gpio_pin: u8) -> Receiver<EnvironmentData> {
    let (tx, rx) = channel::<EnvironmentData>();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(READ_WAIT_SEC));

        let read = dht22_pi::read(gpio_pin);
        match read {
            Ok(val) => {
                log::debug!("reading: {:?}", val);
                match tx.send(val) {
                    Err(e) => {
                        log::debug!("error on writing to channel: {:?}", e);
                        return;
                    }
                    _ => {
                        log::debug!("data sent to channel");
                    }
                }
            }
            Err(e) => {
                log::debug!("reading error: {:?}", e);
            }
        }
    });

    rx
}
