use crate::config::Configuration;
use crate::device::Device;
use crate::rx::RxMode;
use crate::tx::TxMode;
use core::fmt;

/// Represents **Standby-I** mode
///
/// This represents the state the device is in inbetween TX or RX
/// mode.
pub struct StandbyMode<D: Device> {
    device: D,
}

impl<D: Device> fmt::Debug for StandbyMode<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StandbyMode")
    }
}

impl<D: Device> StandbyMode<D> {
    pub fn new(mut device: D) -> Self {
        device.ce_disable();

        StandbyMode { device }
    }

    pub fn rx(self) -> Result<RxMode<D>, D::Error> {
        RxMode::new(self.device)
    }

    /// Go into TX mode
    pub fn tx(self) -> Result<TxMode<D>, D::Error> {
        TxMode::new(self.device)
    }


    pub fn power_down(self) -> Result<D, D::Error> {
        let mut device = self.device;

        device.update_config(|config| config.set_pwr_up(false))
            .map(|_| device)
    }
}

impl<D: Device> Configuration for StandbyMode<D> {
    type Inner = D;
    fn device(&mut self) -> &mut Self::Inner {
        &mut self.device
    }
}
