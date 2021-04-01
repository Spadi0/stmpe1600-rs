use crate::Error;
use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Read, Write};

const DEVICE_ID: u16 = 0x1600;

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
/// The different adresses of the registers on the STMPE1600's I²C bus.
pub enum Register {
	/// ID unique to the STMPE1600
	ChipID = 0x00,
	/// Reset and interrupt control
	SystemControl = 0x03,
	/// GPIO interrupt enable register
	IEGPIOR = 0x08,
	/// GPIO interrupt status register
	ISGPIOR = 0x0A,
	/// GPIO monitor pin state register
	GPMR = 0x10,
	/// GPIO set pin state register
	GPSR = 0x12,
	/// GPIO set pin direction register
	GPDR = 0x14,
	/// GPIO polarity inversion register
	GPPIR = 0x16,
}

#[derive(Debug)]
pub(crate) struct Stmpe1600Device<I2C> {
	i2c: I2C,
	address: u8,
}

impl<I2C, E> Stmpe1600Device<I2C>
where
	I2C: Read<Error = E> + Write<Error = E>,
	E: Debug,
{
	pub fn new(i2c: I2C, address: u8) -> Result<Stmpe1600Device<I2C>, Error<E>> {
		let mut device = Stmpe1600Device { i2c, address };
		device.init()?;
		Ok(device)
	}

	pub fn read_reg(&mut self, register: Register) -> Result<u16, Error<E>> {
		self.i2c
			.write(self.address, &[register as u8])
			.map_err(Error::I2CError)?;
		let mut buffer = [0u8; 2];
		self.i2c
			.read(self.address, &mut buffer)
			.map_err(Error::I2CError)?;
		Ok((buffer[1] as u16) << 8 | buffer[0] as u16)
	}

	pub fn read_reg8(&mut self, register: Register) -> Result<u8, Error<E>> {
		self.i2c
			.write(self.address, &[register as u8])
			.map_err(Error::I2CError)?;
		let mut buffer = [0u8];
		self.i2c
			.read(self.address, &mut buffer)
			.map_err(Error::I2CError)?;
		Ok(buffer[0])
	}

	pub fn write_reg(&mut self, register: Register, value: u16) -> Result<(), Error<E>> {
		self.i2c
			.write(
				self.address,
				&[register as u8, value as u8, (value >> 8) as u8],
			)
			.map_err(Error::I2CError)
	}

	pub fn write_reg8(&mut self, register: Register, value: u8) -> Result<(), Error<E>> {
		self.i2c
			.write(self.address, &[register as u8, value as u8])
			.map_err(Error::I2CError)
	}

	pub fn get_interrupts(&mut self) -> Result<[bool; 16], Error<E>> {
		let mask = self.read_reg(Register::ISGPIOR)?;
		let mut arr = [false; 16];
		for i in 0..16 {
			if mask & 1 << i == 1 << i {
				arr[i] = true;
			}
		}
		Ok(arr)
	}

	fn init(&mut self) -> Result<(), Error<E>> {
		if self.read_reg(Register::ChipID)? != DEVICE_ID {
			return Err(Error::InvalidDeviceID);
		}

		// Do a software reset
		self.write_reg8(Register::SystemControl, 0x80)?;

		Ok(())
	}
}
