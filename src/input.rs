use anyhow::Context as _;
use evdev_rs::{enums, Device, InputEvent, ReadFlag, ReadStatus};
use glob::glob;
use nix::errno::Errno;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn listen(file: File, handler: Sender<i64>) -> anyhow::Result<()> {
    let device = Device::new_from_fd(file).with_context(|| "")?;

    let mut ev: Result<(ReadStatus, InputEvent), Errno>;
    loop {
        ev = device.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING);
        match ev {
            Ok(result) => match result.0 {
                ReadStatus::Sync => {
                    warn!("dropped");
                    loop {
                        if let Ok(event) = device.next_event(ReadFlag::SYNC) {
                            if event.0 == ReadStatus::Sync {
                                warn!("re-synced");
                                break;
                            }
                        }
                    }
                }
                ReadStatus::Success => {
                    let event = result.1;
                    if event.is_type(&enums::EventType::EV_KEY) && event.value == 1 {
                        debug!(
                            "{}: {}: {}",
                            event.event_type, event.event_code, event.value
                        );
                        handler.send(1).unwrap()
                    }
                }
            },
            Err(e) => debug!("Failed to get event. errno: {}", e),
        }
    }
}

pub fn devices() -> anyhow::Result<Vec<(PathBuf, Device)>> {
    let mut devices = vec![];
    let events = glob("/dev/input/event*").context("Failed glob /dev/input/event*")?;

    for entry in events {
        let path = entry.context("Failed to get event file")?;
        let file = File::open(&path).context("Failed to open event file")?;
        let device = Device::new_from_fd(file)
            .context(format!("Failed to get device: {}", &path.display()))?;
        if is_keyboard_device(&device) {
            debug!("{}", device.name().unwrap());
            devices.push((path, device));
        }
    }

    Ok(devices)
}

pub fn select_device() -> anyhow::Result<PathBuf> {
    let devices = devices()?;
    devices
        .first()
        .map(|v| v.0.clone())
        .ok_or(anyhow!("device not found"))
}

fn is_keyboard_device(device: &Device) -> bool {
    let support_enter_key = device.has(&enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ENTER));
    let support_a_key = device.has(&enums::EventCode::EV_KEY(enums::EV_KEY::KEY_A));
    let support_z_key = device.has(&enums::EventCode::EV_KEY(enums::EV_KEY::KEY_Z));
    let support_btn_mouse = device.has(&enums::EventCode::EV_KEY(enums::EV_KEY::BTN_LEFT));
    let is_support = support_enter_key && support_a_key && support_z_key && !support_btn_mouse;
    debug!("{}: keyboard => {}", device.name().unwrap(), is_support);
    is_support
}
