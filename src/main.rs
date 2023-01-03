extern crate dotenv_codegen;

use dotenv_codegen::dotenv;
use signal_hook::consts::{SIGINT, SIGTERM, SIGUSR1};
use switch::{Switch, SwitchState};

use std::error::Error;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use dotenv::dotenv;

mod sensor;
mod switch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_env().unwrap();
    dotenv().ok();

    let switch_mac: &str = dotenv!("SWITCH_PLUG_MAC");
    let is_debug = dotenv!("PROFILE") == "debug";
    let mut switch = switch::new(switch_mac, is_debug);

    let gpio_pin: u8 = dotenv!("SENSOR_PIN")
        .parse()
        .expect("must be a valid number");
    let receiver = sensor::start_read(gpio_pin);

    const LOOP_WAIT: u64 = 2;
    let upper_temp: f32 = dotenv!("UPPER_TEMP")
        .parse()
        .expect("must be a valid float number");
    let lower_temp: f32 = dotenv!("LOWER_TEMP")
        .parse()
        .expect("must be a valid float number");

    assert!(
        upper_temp >= lower_temp,
        "shouldn't be UPPER_TEMP < LOWER_TEMP"
    );

    let mut expect_state: Option<SwitchState> = None;

    let term_sig = Arc::new(AtomicBool::new(false));
    let output_read = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(SIGINT, Arc::clone(&term_sig)).unwrap();
    signal_hook::flag::register(SIGTERM, Arc::clone(&term_sig)).unwrap();
    signal_hook::flag::register(SIGUSR1, Arc::clone(&output_read)).unwrap();

    while !term_sig.load(std::sync::atomic::Ordering::Relaxed) {
        sleep(Duration::from_secs(LOOP_WAIT));

        if let Some(val) = receiver.recv().ok() {
            log::debug!("read data : {val:?} vs {lower_temp:?} / {upper_temp:?}");
            if output_read.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("read data : {val:?} vs {lower_temp:?} / {upper_temp:?}");
            }

            if val.temperature <= lower_temp {
                expect_state = switch_on(&mut switch, expect_state, Arc::clone(&output_read)).await;
            }

            if val.temperature >= upper_temp {
                expect_state =
                    switch_off(&mut switch, expect_state, Arc::clone(&output_read)).await;
            }

            output_read.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        log::debug!("done loop");
    }

    log::info!("will shutdown");
    drop(receiver);
    switch.disconnect().await;
    Ok(())
}

async fn switch_on(
    switch: &mut Switch,
    expect_state: Option<SwitchState>,
    output_read: Arc<AtomicBool>,
) -> Option<SwitchState> {
    match expect_state {
        Some(SwitchState::ON) => {
            if output_read.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("no need to action");
            }
            return expect_state;
        }
        _ => {}
    }

    match switch.setup().await {
        Err(e) => {
            log::warn!("BLE setup error {e:?}");
            switch.disconnect().await;
            return expect_state;
        }
        _ => {}
    }

    if let Some(switch_state_now) = switch.get_current_state().await {
        log::debug!("switch state data : {switch_state_now:?}");

        if switch_state_now == switch::SwitchState::OFF {
            log::debug!("trigger on");
            switch.send_on_cmd().await;
        }
    }

    switch.disconnect().await;
    Some(SwitchState::ON)
}

async fn switch_off(
    switch: &mut Switch,
    expect_state: Option<SwitchState>,
    output_read: Arc<AtomicBool>,
) -> Option<SwitchState> {
    match expect_state {
        Some(SwitchState::OFF) => {
            if output_read.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("no need to action");
            }

            return expect_state;
        }
        _ => {}
    }

    match switch.setup().await {
        Err(e) => {
            log::warn!("BLE setup error {e:?}");
            switch.disconnect().await;
            return expect_state;
        }
        _ => {}
    }

    if let Some(switch_state_now) = switch.get_current_state().await {
        log::debug!("switch state data : {switch_state_now:?}");

        if switch_state_now == switch::SwitchState::ON {
            log::debug!("trigger off");
            switch.send_off_cmd().await;
        }
    }

    switch.disconnect().await;
    Some(SwitchState::OFF)
}
