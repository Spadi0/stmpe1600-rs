use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Read, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use crate::{Error, PinMode, Register, Stmpe1600};

/// A single I/O pin on the STMPE1600.
/// 
/// These implement the `embedded-hal` traits for GPIO pins (`InputPin` and `OutputPin`),
/// so they can be used to transparently connect devices driven over GPIO pins through the STMPE1600 instead, using any
/// `embedded-hal` compatible device drivers without modification.
pub struct Pin<'a, I2C> {
	driver: &'a Stmpe1600<I2C>,
	pin_number: u8,
}

impl<'a, I2C, E> Pin<'a, I2C>
	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
{
	pub(crate) fn new(driver: &'a Stmpe1600<I2C>, pin_number: u8) -> Pin<'a, I2C> {
		Pin { driver, pin_number }
	}
}

impl<'a, I2C, E> InputPin for Pin<'a, I2C>
	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
{
	type Error = Error<E>;

	fn is_low(&self) -> Result<bool, Self::Error> {
		if self.driver.pins[self.pin_number as usize] == PinMode::Output {
			return Err(Error::IncorrectPinMode);
		}

		let mask = self.driver.device.borrow_mut().read_reg(Register::GPMR)?;
		Ok(mask & (1 << self.pin_number) == 0)
	}

	fn is_high(&self) -> Result<bool, Self::Error> {
		if self.driver.pins[self.pin_number as usize] == PinMode::Output {
			return Err(Error::IncorrectPinMode);
		}

		let mask = self.driver.device.borrow_mut().read_reg(Register::GPMR)?;
		Ok(mask & (1 << self.pin_number) == 1 << self.pin_number)
	}
}

impl<'a, I2C, E> OutputPin for Pin<'a, I2C>
	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
{
	type Error = Error<E>;

	fn set_low(&mut self) -> Result<(), Self::Error> {
		if self.driver.pins[self.pin_number as usize] != PinMode::Output {
			return Err(Error::IncorrectPinMode);
		}

		let mask = self.driver.device.borrow_mut().read_reg(Register::GPSR)?;
		self.driver.device.borrow_mut().write_reg(Register::GPSR, mask & !(1 << self.pin_number))
	}

	fn set_high(&mut self) -> Result<(), Self::Error> {
		if self.driver.pins[self.pin_number as usize] != PinMode::Output {
			return Err(Error::IncorrectPinMode);
		}

		let mask = self.driver.device.borrow_mut().read_reg(Register::GPSR)?;
		self.driver.device.borrow_mut().write_reg(Register::GPSR, mask | (1 << self.pin_number))
	}
}