use tokio::sync::mpsc;

mod devices;

use crate::telemetry::Telemetry;

pub enum HwBoundEvent {
    UpdateTelemetry(Telemetry),
    RequestDeviceList,
}

pub enum AppBoundEvent {
    UpdateDeviceList(Vec<devices::Device>),
}

pub async fn main(mut rx: mpsc::Receiver<HwBoundEvent>, tx: mpsc::Sender<AppBoundEvent>) {
    let api = hidapi::HidApi::new().expect("Failed to construct HidApi!");
    let mut device_list = Vec::new();

    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                if let Some(event) = maybe_event {
                    match event {
                        HwBoundEvent::RequestDeviceList => {
                            device_list = get_device_list(&api);
                            // tx.send(AppBoundEvent::UpdateDeviceList(device_list_ids)).await;
                        },
                        HwBoundEvent::UpdateTelemetry(v) => {
                            for device in &device_list {
                                let _ = match device {
                                    devices::Device::RpmGauge(rpm_gauge) => rpm_gauge.update_rpm(v.engine.rpm as u16),
                                };
                            }
                        },
                    }
                }
            },
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(150)) => {
                let mut new_device_list = Vec::new();
                for device in device_list.drain(..) {
                    let result = match &device {
                        devices::Device::RpmGauge(rpm_gauge) => rpm_gauge.heartbeat(),
                    };
                    if let Err(e) = result {
                        error!("Lost connection to device! Error: {:?}", e);
                    } else {
                        new_device_list.push(device);
                    }
                }
                device_list = new_device_list;
            },
        }
    }
}

fn get_device_list(api: &hidapi::HidApi) -> Vec<devices::Device> {
    const SUPPORTED_VID: [u16; 1] = [
        6991,
    ];

    const SUPPORTED_PID: [u16; 1] = [
        37382,
    ];

    let mut device_list = Vec::new();

    for hid_device in api.device_list() {
        // println!("{:#?}", device);
        // println!("{:?} ({:?}): {:?}", device.manufacturer_string(), device.product_string(), device);

        // Now we have to detect any device that is currently supported.
        let vid = hid_device.vendor_id();
        let pid = hid_device.product_id();

        if let hidapi::BusType::Usb = hid_device.bus_type() {
            if SUPPORTED_VID.contains(&vid) && SUPPORTED_PID.contains(&pid) {
                // We found a supported device
                match devices::Device::from_hid_device(api, vid, pid) {
                    Ok(device) => device_list.push(device),
                    Err(e) => error!("Error trying to load device ({vid}:{pid}): {:?}", e),
                }
            }
        }
    }

    device_list
}
