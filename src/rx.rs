use crate::command::{ReadRxPayload, ReadRxPayloadWidth};
use crate::config::Configuration;
use crate::device::Device;
use crate::payload::Payload;
use crate::registers::FifoStatus;
use crate::standby::StandbyMode;
use core::fmt;

pub struct RxMode<D: Device> {
    device: D,
}

impl<D: Device> fmt::Debug for RxMode<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RxMode")
    }
}

impl<D: Device> RxMode<D> {
    pub(crate) fn new(mut device: D) -> Result<Self, D::Error> {
        device.update_config(|config| config.set_prim_rx(true))?;
        device.ce_enable();

        Ok(RxMode { device })
    }

    pub fn standby(self) -> StandbyMode<D> {
        StandbyMode::new(self.device)
    }

    /// Is there any incoming data to read? Return the pipe number.
    pub fn can_read(&mut self) -> Result<Option<u8>, D::Error> {
        self.device
            .read_register::<FifoStatus>()
            .map(|(status, fifo_status)| {
                if !fifo_status.rx_empty() {
                    Some(status.rx_p_no())
                } else {
                    None
                }
            })
    }

    /// Is the RX queue empty?
    pub fn is_empty(&mut self) -> Result<bool, D::Error> {
        self.device
            .read_register::<FifoStatus>()
            .map(|(_, fifo_status)| fifo_status.rx_empty())
    }

    /// Is the RX queue full?
    pub fn is_full(&mut self) -> Result<bool, D::Error> {
        self.device
            .read_register::<FifoStatus>()
            .map(|(_, fifo_status)| fifo_status.rx_full())
    }

    pub fn read(&mut self) -> Result<Payload, D::Error> {
        let (_, payload_width) = self.device.send_command(&ReadRxPayloadWidth)?;
        let (_, payload) = self
            .device
            .send_command(&ReadRxPayload::new(payload_width as usize))?;
        Ok(payload)
    }
}

impl<D: Device> Configuration for RxMode<D> {
    type Inner = D;
    fn device(&mut self) -> &mut Self::Inner {
        &mut self.device
    }
}
