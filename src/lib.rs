// Copyright 2018, Astro <astro@spaceboyz.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE>. This file
// may not be copied, modified, or distributed except according to
// those terms.

//#![no_std]
extern crate embedded_hal;
#[macro_use]
extern crate bitfield;

use core::fmt;
use core::fmt::Debug;
use embedded_hal::blocking::spi::Transfer as SpiTransfer;
use embedded_hal::digital::OutputPin;

mod config;

pub use crate::config::{Configuration, CrcMode, DataRate};

pub mod setup;

mod registers;

pub use crate::registers::{Config, Register, SetupAw, FifoStatus, Status};

mod command;

use crate::command::{Command, ReadRegister, WriteRegister};

mod payload;

pub use crate::payload::Payload;

mod errors;

pub use crate::errors::{Error, TransmissionError};

mod device;

pub use crate::device::Device;

mod standby;

pub use crate::standby::StandbyMode;

mod rx;

pub use crate::rx::RxMode;

mod tx;

pub use crate::tx::TxMode;

pub const PIPES_COUNT: usize = 6;
pub const MIN_ADDR_BYTES: usize = 3;
pub const MAX_ADDR_BYTES: usize = 5;

fn get_initial_config() -> Config {
    // Reset Configuration 00001000 is reset word
    let mut config = Config(0b0000_1000);

    config.set_mask_rx_dr(true);
    config.set_mask_tx_ds(true);
    config.set_mask_max_rt(true);

    config
}

fn get_initial_interrupts() -> Status {
    let mut clear = Status(0);

    // Clear TX interrupts by setting bit to '1'
    clear.set_rx_dr(true);
    clear.set_tx_ds(true);
    clear.set_max_rt(true);

    clear
}


/// Driver for the nRF24L01+
pub struct NRF24L01<'a, CE: OutputPin, CSN: OutputPin, SPI: SpiTransfer<u8>> {
    ce: &'a mut CE,
    csn: &'a mut CSN,
    spi: &'a mut SPI,
    config: Config,
}

impl<'a, CE: OutputPin, CSN: OutputPin, SPI: SpiTransfer<u8, Error=SPIE>, SPIE: Debug> fmt::Debug for NRF24L01<'a, CE, CSN, SPI> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NRF24L01")
    }
}

impl<'a, CE: OutputPin, CSN: OutputPin, SPI: SpiTransfer<u8, Error=SPIE>, SPIE: Debug> NRF24L01<'a, CE, CSN, SPI> {
    /// Construct a new driver instance.
    pub fn new(ce: &'a mut CE, csn: &'a mut CSN, spi: &'a mut SPI) -> Self {
        let config = get_initial_config();

        NRF24L01 {
            ce,
            csn,
            spi,
            config,
        }
    }

    pub fn is_connected(&mut self) -> Result<bool, Error<SPIE>> {
        let (_, setup_aw) = self.read_register::<SetupAw>()?;

        let valid = setup_aw.aw() >= 3 && setup_aw.aw() <= 5;

        Ok(valid)
    }

    pub fn power_up(mut self) -> Result<StandbyMode<Self>, Error<SPIE>> {
        self.initialise_device()?;

        assert!(self.is_connected().unwrap());

        self.update_config(|config| config.set_pwr_up(true))
            .map(|_| StandbyMode::new(self))
    }

    fn initialise_device(&mut self) -> Result<(), Error<SPIE>> {
        // Initialise pins
        self.initialise_pins();

        // Initialise config
        self.write_register(get_initial_config())?;

        // Clear Interrupts
        self.write_register(get_initial_interrupts())?;

        Ok(())
    }

    fn initialise_pins(&mut self) {
        self.ce.set_low();
        self.csn.set_high();
    }
}

impl<'a, CE: OutputPin, CSN: OutputPin, SPI: SpiTransfer<u8, Error=SPIE>, SPIE: Debug> Device for NRF24L01<'a, CE, CSN, SPI> {
    type Error = Error<SPIE>;

    fn ce_enable(&mut self) {
        self.ce.set_high();
    }

    fn ce_disable(&mut self) {
        self.ce.set_low();
    }

    fn send_command<C: Command>(&mut self, command: &C) -> Result<(Status, C::Response), Self::Error> {
        // Allocate storage
        let mut buf_storage = [0; 33];
        let len = command.len();
        let buf = &mut buf_storage[0..len];

        // Serialize the command
        command.encode(buf);

        // SPI transaction
        self.csn.set_low();
        let transfer_result = self.spi.transfer(buf);
        self.csn.set_high();

        // Propagate Err only after csn.set_high():
        let result = transfer_result?;

        // Parse response
        let status = Status(result[0]);
        let response = C::decode_response(result);

        Ok((status, response))
    }

    fn write_register<R: Register>(&mut self, register: R) -> Result<Status, Self::Error> {
        self.send_command(&WriteRegister::new(register))
            .map(|(s, _)| s)
    }

    fn read_register<R: Register>(&mut self) -> Result<(Status, R), Self::Error> {
        self.send_command(&ReadRegister::new())
    }

    fn update_config<F, R>(&mut self, f: F) -> Result<R, Self::Error>
        where F: FnOnce(&mut Config) -> R {
        // Mutate
        let old_config = self.config.clone();
        let result = f(&mut self.config);

        if self.config != old_config {
            let config = self.config.clone();
            self.write_register(config)?;
        }
        Ok(result)
    }
}
