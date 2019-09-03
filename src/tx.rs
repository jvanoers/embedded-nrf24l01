use crate::command::WriteTxPayload;
use crate::config::Configuration;
use crate::device::Device;
use crate::registers::{FifoStatus, ObserveTx, Status};
use crate::standby::StandbyMode;

use std::thread::sleep;
use std::time::Duration;
use core::fmt;

use crate::errors::TransmissionError;

/// Represents **TX Mode** and the associated **TX Settling** and
/// **Standby-II** states
///
/// **" It is important to never keep the nRF24L01 in TX mode for more
/// than 4ms at a time."**
pub struct TxMode<D: Device> {
    device: D,
}

impl<D: Device> fmt::Debug for TxMode<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TxMode")
    }
}

impl<D: Device> TxMode<D> {
    pub(crate) fn new(mut device: D) -> Result<Self, D::Error> {
        device.ce_disable();
        device.update_config(|config| config.set_prim_rx(false))?;

        Ok(TxMode { device })
    }

    pub fn standby(mut self) -> Result<StandbyMode<D>, D::Error> {
        self.flush_queue()
            .or(self.flush_tx())
            .map(|_| StandbyMode::new(self.device))
    }

    /// Is TX FIFO empty?
    pub fn is_empty(&mut self) -> Result<bool, D::Error> {
        let (_, fifo_status) = self.device.read_register::<FifoStatus>()?;
        Ok(fifo_status.tx_empty())
    }

    /// Is TX FIFO full?
    pub fn is_full(&mut self) -> Result<bool, D::Error> {
        let (_, fifo_status) = self.device.read_register::<FifoStatus>()?;
        Ok(fifo_status.tx_full())
    }

    /// Send asynchronously
    pub fn enqueue(&mut self, packet: &[u8]) -> Result<Status, D::Error> {
        // TODO Guarantee packet length is <= 32
        // TODO Ensure queue is not full

        self.device.send_command(&WriteTxPayload::new(packet))
            .map(|(s, _)| s)
    }

    pub fn flush_queue(&mut self) -> Result<(), TransmissionError<D::Error>> {
        // CE enabled for >10uS Initiates transmission
        self.device.ce_enable();

        sleep(Duration::new(0, 10_000));

        let mut empty = false;

        while !empty {
            let (status, fifo_status) = self.device.read_register::<FifoStatus>()?;

            empty = fifo_status.tx_empty();

            // MAX_RT: Maximum retransmissions interrupt, tranmission stops
            if status.max_rt() {
                self.device.ce_disable();

                let mut clear = Status(0);
                // Clear TX interrupts
                clear.set_tx_ds(true);
                clear.set_max_rt(true);

                self.device.write_register(clear)?;

                self.device.ce_disable();

                return Err(TransmissionError::MaximumRetriesExceeded);
            }
        }

        self.device.ce_disable();

        Ok(())
    }

    pub fn observe(&mut self) -> Result<ObserveTx, D::Error> {
        self.device.read_register()
            .map(|(_, observe_tx)| observe_tx)
    }
}

impl<D: Device> Configuration for TxMode<D> {
    type Inner = D;
    fn device(&mut self) -> &mut Self::Inner {
        &mut self.device
    }
}
