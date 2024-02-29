use log::info;

pub struct Device {}

impl Device {
    pub fn new() -> Self {
        info!("No device yet");
        Self {}
    }
}