use hidapi::HidDevice;

pub struct RpmGauge {
    device: HidDevice,
    max_rpm: u16,
}

impl RpmGauge {
    pub fn new(device: HidDevice, max_rpm: u16) -> Self {
        Self {
            device,
            max_rpm,
        }
    }

    pub fn heartbeat(&self) -> anyhow::Result<()> {
        let mut data: [u8; 2] = [0; 2];
        data[1] = 1;
        self.device.write(&data)?;
        Ok(())
    }

    pub fn update_rpm(&self, rpm: u16) -> anyhow::Result<()> {
        let rpm = rpm.min(self.max_rpm);
        let mut data: [u8; 4] = [0; 4];
        data[1] = 2;
        data[2] = rpm as u8;
        data[3] = (rpm >> 8) as u8;
        self.device.write(&data)?;
        Ok(())
    }
}
