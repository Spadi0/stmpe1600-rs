//! Platform-agnostic driver for the STMPE1600.

#![no_std]

#[macro_use]
extern crate bitflags;

use core::fmt::Debug;
use core::cell::RefCell;
use embedded_hal::blocking::i2c::{Read, Write};

/// The default I²C address for the STMPE1600.
pub const DEFAULT_ADDRESS: u8 = 0x42;

const DEVICE_ID: [u8; 2] = [0x00, 0x16];

/// STMPE1600 device driver
#[derive(Debug)]
pub struct Stmpe1600<I2C> {
	device: RefCell<Stmpe1600Data<I2C>>
}

#[derive(Debug)]
pub(crate) struct Stmpe1600Data<I2C> {
	address: u8,
	i2c: I2C,
	callbacks: [Option<fn()>; 16],
}

bitflags! {
	/// All the available pins on the STMPE1600.
	pub struct PinFlag: u16 {
		const PIN0 = 0x0001;
		const PIN1 = 0x0002;
		const PIN2 = 0x0004;
		const PIN3 = 0x0008;
		const PIN4 = 0x0010;
		const PIN5 = 0x0020;
		const PIN6 = 0x0040;
		const PIN7 = 0x0080;
		const PIN8 = 0x0100;
		const PIN9 = 0x0200;
		const PIN10 = 0x0400;
		const PIN11 = 0x0800;
		const PIN12 = 0x1000;
		const PIN13 = 0x2000;
		const PIN14 = 0x4000;
		const PIN15 = 0x8000;
	}
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
pub(crate) enum Register {
	ChipID = 0x00,
	SystemControl = 0x03,
	IEGPIOR = 0x08,
	ISGPIOR = 0x0A,
	GPMR = 0x10,
	GPSR = 0x12,
	GPDR = 0x14,
	GPPIR = 0x16,
}

/// All the different types of errors that can occur while interacting with the STMPE1600.
#[derive(Debug)]
pub enum Error<E>
	where E: Debug
{
	/// I²C bus error
	I2CError(E),
	/// Invalid device ID
	InvalidDeviceID,
}

impl<I2C, E> Stmpe1600<I2C> 
	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
{
	/// Creates a new instance of the STMPE1600 device with the default address.
	pub fn new(i2c: I2C) -> Result<Self, Error<E>> {
		Self::new_with_address(i2c, DEFAULT_ADDRESS)
	}

	/// Creates a new instance of the STMPE1600 device with a specified address.
	pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, Error<E>> {
		let stmpe1600 = Self {
			device: RefCell::new(
				Stmpe1600Data {
					address,
					i2c,
					callbacks: [None; 16]
				}
			)
		};

		if stmpe1600.get_device_id()? != DEVICE_ID {
			return Err(Error::InvalidDeviceID);
		}

		// Do a software reset
		stmpe1600.write_reg(Register::SystemControl, 0x80)?;
		// Enable interrupts
		stmpe1600.write_reg(Register::SystemControl, 0x04)?;
		// Clear the input pin status register
		stmpe1600.get_gpmr()?;

		Ok(stmpe1600)
	}

	/// Destroy the driver instance and return the I²C bus instance.
	pub fn destroy(self) -> I2C {
		self.device.into_inner().i2c
	}

	/// Sets the interrupt polarity to active HIGH (rising edge output) or active LOW (falling edge output)
	pub fn set_interrupt_polarity(&self, is_active_high: bool) -> Result<(), Error<E>> {
		let mut buf = [0u8];
		self.read_reg(Register::SystemControl, &mut buf)?;
		if is_active_high {
			self.write_reg(Register::SystemControl, buf[0] & !0b1)
		} else {
			self.write_reg(Register::SystemControl, buf[0] | 0b1)
		}
	}

	/// Setup the specified pins as input pins.
	pub fn setup_input_pins(&self, pins: PinFlag) -> Result<(), Error<E>> {
		let mask = pins.bits();
		let mut reg = [0u8; 2];

		self.read_reg(Register::GPDR, &mut reg)?;
		self.write_reg16(Register::GPDR, ((reg[1] as u16) << 8 | reg[0] as u16) & !mask)?;

		Ok(())
	}

	/// Setup the specified pins as output pins.
	pub fn setup_output_pins(&self, pins: PinFlag) -> Result<(), Error<E>> {
		let mask = pins.bits();
		let mut reg = [0u8; 2];

		self.read_reg(Register::GPDR, &mut reg)?;
		self.write_reg16(Register::GPDR, ((reg[1] as u16) << 8 | reg[0] as u16) | mask)?;

		Ok(())
	}

	/// Setup the specified pins as interrupt pins.
	pub fn setup_interrupt_pins(&self, pins: PinFlag, callback: fn()) -> Result<(), Error<E>> {
		let mask = pins.bits();
		let mut reg = [0u8; 2];

		self.read_reg(Register::GPDR, &mut reg)?;
		self.write_reg16(Register::GPDR, ((reg[1] as u16) << 8 | reg[0] as u16) & !mask)?;
		self.read_reg(Register::IEGPIOR, &mut reg)?;
		self.write_reg16(Register::IEGPIOR, ((reg[1] as u16) << 8 | reg[0] as u16) | mask)?;

		for i in 0..16 {
			if mask & (1 << i) != 0 {
				self.device.borrow_mut().callbacks[i] = Some(callback);
			}
		}

		Ok(())
	}

	/// Gets the state of all the pins.
	pub fn get_state(&self) -> Result<u16, Error<E>> {
		self.get_gpmr()
	}

	/// Sets the state of all the pins.
	pub fn set_state(&self, pins: PinFlag) -> Result<(), Error<E>> {
		self.write_reg16(Register::GPSR, pins.bits())
	}

	/// Checks all the interrupt lines, and if there is an interrupt, the corresponding callback is executed.
	pub fn handle_interrupt(&self) -> Result<(), Error<E>> {
		let isgpior = self.get_isgpior()?;
		for i in 0..16 {
			if let Some(callback) = self.device.borrow().callbacks[i] {
				if isgpior & (1 << i) != 0 {
					callback();
				}
			}
		}

		Ok(())
	}

	fn read_reg(&self, register: Register, buffer: &mut [u8]) -> Result<(), Error<E>> {
		let mut device = self.device.borrow_mut();
		let address = device.address;
		device.i2c.write(address, &[register as u8]).map_err(|err| Error::I2CError(err))?;
		device.i2c.read(address, buffer).map_err(|err| Error::I2CError(err))
	}

	fn write_reg(&self, register: Register, value: u8) -> Result<(), Error<E>> {
		let mut device = self.device.borrow_mut();
		let address = device.address;
		device.i2c.write(address, &[register as u8, value]).map_err(|err| Error::I2CError(err))
	}

	fn write_reg16(&self, register: Register, value: u16) -> Result<(), Error<E>> {
		let mut device = self.device.borrow_mut();
		let address = device.address;
		device.i2c.write(address, &[register as u8, value as u8, (value >> 8) as u8]).map_err(|err| Error::I2CError(err))
	}

	fn get_device_id(&self) -> Result<[u8; 2], Error<E>> {
		let mut device_id = [0u8; 2];
		self.read_reg(Register::ChipID, &mut device_id)?;
		Ok(device_id)
	}

	fn get_isgpior(&self) -> Result<u16, Error<E>> {
		let mut buf = [0u8; 2];
		self.read_reg(Register::ISGPIOR, &mut buf)?;
		Ok((buf[1] as u16) << 8 | buf[0] as u16)
	}

	fn get_gpmr(&self) -> Result<u16, Error<E>> {
		let mut buf = [0u8; 2];
		self.read_reg(Register::GPMR, &mut buf)?;
		Ok((buf[1] as u16) << 8 | buf[0] as u16)
	}
}
