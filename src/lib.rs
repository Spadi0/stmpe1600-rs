//! Platform-agnostic driver for the STMPE1600.

#![no_std]

use core::fmt::Debug;
use core::cell::RefCell;
use embedded_hal::blocking::i2c::{Read, Write};

mod builder;
use builder::Stmpe1600Builder;
mod device;
use device::{Register, Stmpe1600Device};

/// The default I²C address for the STMPE1600.
pub const DEFAULT_ADDRESS: u8 = 0x42;

/// STMPE1600 device driver
#[derive(Debug)]
pub struct Stmpe1600<I2C> {
	device: RefCell<Stmpe1600Device<I2C>>
}

/// The types that the pins on the STMPE1600 may be configured as.
#[derive(Clone, Copy, Debug)]
pub enum PinMode {
	Input,
	Output,
	Interrupt
}

/// Tells the STMPE1600 what polarity the interrupt output pin should be when an interrupt is triggered.
#[derive(Clone, Copy, Debug)]
pub enum InterruptPolarity {
	Low,
	High,
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
	pub fn builder(i2c: I2C) -> Stmpe1600Builder<I2C> {
		Stmpe1600Builder::new(i2c)
	}

	pub fn get_interrupts(&self) -> Result<[bool; 16], Error<E>> {
		let mask = self.device.borrow_mut().read_reg(Register::ISGPIOR)?;
		let mut arr = [false; 16];
		for i in 0..16 {
			if mask & 1 << i == 1 << i {
				arr[i] = true;
			}
		}
		Ok(arr)
	}
}

// impl<I2C, E> Stmpe1600<I2C> 
// 	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
// {
// 	/// Creates a new instance of the STMPE1600 device with the default address.
// 	pub fn new(i2c: I2C) -> Result<Self, Error<E>> {
// 		Self::new_with_address(i2c, DEFAULT_ADDRESS)
// 	}

// 	/// Creates a new instance of the STMPE1600 device with a specified address.
// 	pub fn new_with_address(i2c: I2C, address: u8) -> Result<Self, Error<E>> {
// 		let stmpe1600 = Self {
// 			device: RefCell::new(
// 				Stmpe1600Data {
// 					address,
// 					i2c,
// 					callbacks: [None; 16]
// 				}
// 			)
// 		};

// 		if stmpe1600.get_device_id()? != DEVICE_ID {
// 			return Err(Error::InvalidDeviceID);
// 		}

// 		// Do a software reset
// 		stmpe1600.write_reg(Register::SystemControl, 0x80)?;
// 		// Enable interrupts
// 		stmpe1600.write_reg(Register::SystemControl, 0x04)?;
// 		// Clear the input pin status register
// 		stmpe1600.get_gpmr()?;

// 		Ok(stmpe1600)
// 	}

// 	/// Destroy the driver instance and return the I²C bus instance.
// 	pub fn destroy(self) -> I2C {
// 		self.device.into_inner().i2c
// 	}

// 	/// Sets the interrupt polarity to active HIGH (rising edge output) or active LOW (falling edge output)
// 	pub fn set_interrupt_polarity(&self, is_active_high: bool) -> Result<(), Error<E>> {
// 		let mut buf = [0u8];
// 		self.read_reg(Register::SystemControl, &mut buf)?;
// 		if is_active_high {
// 			self.write_reg(Register::SystemControl, buf[0] & !0b1)
// 		} else {
// 			self.write_reg(Register::SystemControl, buf[0] | 0b1)
// 		}
// 	}

// 	/// Setup the specified pins as input pins.
// 	pub fn setup_input_pins(&self, pins: PinFlag) -> Result<(), Error<E>> {
// 		let mask = pins.bits();
// 		let mut reg = [0u8; 2];

// 		self.read_reg(Register::GPDR, &mut reg)?;
// 		self.write_reg16(Register::GPDR, ((reg[1] as u16) << 8 | reg[0] as u16) & !mask)?;

// 		Ok(())
// 	}

// 	/// Setup the specified pins as output pins.
// 	pub fn setup_output_pins(&self, pins: PinFlag) -> Result<(), Error<E>> {
// 		let mask = pins.bits();
// 		let mut reg = [0u8; 2];

// 		self.read_reg(Register::GPDR, &mut reg)?;
// 		self.write_reg16(Register::GPDR, ((reg[1] as u16) << 8 | reg[0] as u16) | mask)?;

// 		Ok(())
// 	}

// 	/// Setup the specified pins as interrupt pins.
// 	pub fn setup_interrupt_pins(&self, pins: PinFlag, callback: fn()) -> Result<(), Error<E>> {
// 		let mask = pins.bits();
// 		let mut reg = [0u8; 2];

// 		self.read_reg(Register::GPDR, &mut reg)?;
// 		self.write_reg16(Register::GPDR, ((reg[1] as u16) << 8 | reg[0] as u16) & !mask)?;
// 		self.read_reg(Register::IEGPIOR, &mut reg)?;
// 		self.write_reg16(Register::IEGPIOR, ((reg[1] as u16) << 8 | reg[0] as u16) | mask)?;

// 		for i in 0..16 {
// 			if mask & (1 << i) != 0 {
// 				self.device.borrow_mut().callbacks[i] = Some(callback);
// 			}
// 		}

// 		Ok(())
// 	}

// 	/// Gets the state of all the pins.
// 	pub fn get_state(&self) -> Result<u16, Error<E>> {
// 		self.get_gpmr()
// 	}

// 	/// Sets the state of all the pins.
// 	pub fn set_state(&self, pins: PinFlag) -> Result<(), Error<E>> {
// 		self.write_reg16(Register::GPSR, pins.bits())
// 	}

// 	/// Checks all the interrupt lines, and if there is an interrupt, the corresponding callback is executed.
// 	pub fn handle_interrupt(&self) -> Result<(), Error<E>> {
// 		let isgpior = self.get_isgpior()?;
// 		for i in 0..16 {
// 			if let Some(callback) = self.device.borrow().callbacks[i] {
// 				if isgpior & (1 << i) != 0 {
// 					callback();
// 				}
// 			}
// 		}

// 		Ok(())
// 	}

// 	fn read_reg(&self, register: Register, buffer: &mut [u8]) -> Result<(), Error<E>> {
// 		let mut device = self.device.borrow_mut();
// 		let address = device.address;
// 		device.i2c.write(address, &[register as u8]).map_err(|err| Error::I2CError(err))?;
// 		device.i2c.read(address, buffer).map_err(|err| Error::I2CError(err))
// 	}

// 	fn write_reg(&self, register: Register, value: u8) -> Result<(), Error<E>> {
// 		let mut device = self.device.borrow_mut();
// 		let address = device.address;
// 		device.i2c.write(address, &[register as u8, value]).map_err(|err| Error::I2CError(err))
// 	}

// 	fn write_reg16(&self, register: Register, value: u16) -> Result<(), Error<E>> {
// 		let mut device = self.device.borrow_mut();
// 		let address = device.address;
// 		device.i2c.write(address, &[register as u8, value as u8, (value >> 8) as u8]).map_err(|err| Error::I2CError(err))
// 	}

// 	fn get_device_id(&self) -> Result<[u8; 2], Error<E>> {
// 		let mut device_id = [0u8; 2];
// 		self.read_reg(Register::ChipID, &mut device_id)?;
// 		Ok(device_id)
// 	}

// 	fn get_isgpior(&self) -> Result<u16, Error<E>> {
// 		let mut buf = [0u8; 2];
// 		self.read_reg(Register::ISGPIOR, &mut buf)?;
// 		Ok((buf[1] as u16) << 8 | buf[0] as u16)
// 	}

// 	fn get_gpmr(&self) -> Result<u16, Error<E>> {
// 		let mut buf = [0u8; 2];
// 		self.read_reg(Register::GPMR, &mut buf)?;
// 		Ok((buf[1] as u16) << 8 | buf[0] as u16)
// 	}
// }
