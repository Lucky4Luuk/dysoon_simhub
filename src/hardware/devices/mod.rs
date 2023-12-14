pub mod rpm_gauge;

pub enum Device {
    RpmGauge(rpm_gauge::RpmGauge),
}

impl Device {
    pub fn from_hid_device(api: &hidapi::HidApi, vid: u16, pid: u16) -> anyhow::Result<Self> {
        const MAGIC: [u8; 5] = [0, 123, 38, 83, 231]; // First zero is the report ID (just 0)

        trace!("Connecting to device {vid}:{pid}");

        let device = api.open(vid, pid)?;
        device.set_blocking_mode(true)?;
        device.write(&MAGIC)?;

        // for _ in 0..32 {
        //     let mut read_buf = [0u8; 5]; // First byte will contain the report ID, which should be 0
        //     let bytes_read = device.read_timeout(&mut read_buf, 150)?;
        //     println!("read_buf (bytes read: {bytes_read}): {:?}", read_buf);
        //     std::thread::sleep(std::time::Duration::from_millis(50));
        // }

        let mut read_buf = [0u8; 64];
        let bytes_read = device.read(&mut read_buf)?;
        if bytes_read >= 4 {
            let device_type = read_buf[0];
            let unit_type = read_buf[1];
            let max_value = (read_buf[2] as u16) | ((read_buf[3] as u16) << 8);
            debug!("device_type: {device_type}");
            debug!("unit_type: {unit_type}");
            debug!("max_value: {max_value}");
            match device_type {
                0 => Ok(Device::RpmGauge(rpm_gauge::RpmGauge::new(device, max_value))),
                _ => Err(InitDeviceError::UnknownDevice.into()),
            }
        } else {
            Err(InitDeviceError::NotEnoughDataRead.into())
        }
    }
}

#[derive(thiserror::Error, Debug)]
/// error initializing the device
pub enum InitDeviceError {
    /// unknown device
    UnknownDevice,
    /// not enough bytes to read
    NotEnoughDataRead,
}

impl std::fmt::Display for InitDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
