use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Read, Write};
use crate::Error;

const DEVICE_ID: u16 = 0x0016;

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

#[derive(Debug)]
pub(crate) struct Stmpe1600Device<I2C> {
	i2c: I2C,
	address: u8,
}

impl<I2C, E> Stmpe1600Device<I2C>
	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
{
	pub fn new(i2c: I2C, address: u8) -> Result<Stmpe1600Device<I2C>, Error<E>> {
		let mut device = Stmpe1600Device { i2c, address };
		device.init()?;
		Ok(device)
	}
	
	pub fn read_reg(&mut self, register: Register) -> Result<u16, Error<E>> {
		self.i2c.write(self.address, &[register as u8]).map_err(Error::I2CError)?;
		let mut buffer = [0u8; 2];
		self.i2c.read(self.address, &mut buffer).map_err(Error::I2CError)?;
		Ok((buffer[1] as u16) << 8 | buffer[0] as u16)
	}

	pub fn write_reg(&mut self, register: Register, value: u16) -> Result<(), Error<E>> {
		self.i2c.write(self.address, &[register as u8, value as u8, (value >> 8) as u8]).map_err(Error::I2CError)
	}

	fn init(&mut self) -> Result<(), Error<E>> {
		if self.read_reg(Register::ChipID)? != DEVICE_ID {
			return Err(Error::InvalidDeviceID);
		}

		// Do a software reset
		self.write_reg(Register::SystemControl, 0x80)?;
		// Clear the input pin status register
		self.read_reg(Register::GPMR)?;

		Ok(())
	}
}