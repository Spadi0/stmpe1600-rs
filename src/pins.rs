use crate::{Error, PinMode, Polarity, Register, Stmpe1600};
use core::marker::PhantomData;
use embedded_hal::blocking::i2c::{Read, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub mod modes {
	pub struct Input;
	pub struct Output;
	pub struct Interrupt;
}
use modes::*;

/// A single I/O pin on the STMPE1600.
///
/// `Pin` takes a `MODE` as a generic argument, which is either `Input`, `Output` or `Interrupt`,
/// and indicates which mode of operation the current pin is configured for. This mode can be
/// changed by using the `into_input_pin`, `into_output_pin` and `into_interrupt_pin` functions
/// respectively.
///
/// Input and interrupt pins implement the trait [`embedded_hal::digital::v2::InputPin`], and output
/// pins implement [`embedded_hal::digital::v2::OutputPin`]. This means that the pins on the I/O
/// expander can be used by platform agnostic drivers as if they were regular GPIO pins.
pub struct Pin<'a, I2C, MODE> {
	driver: &'a Stmpe1600<I2C>,
	pin: u8,
	_phantom: PhantomData<MODE>,
}

impl<'a, E, I2C, MODE> Pin<'a, I2C, MODE>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	pub(crate) fn new(driver: &'a Stmpe1600<I2C>, pin: u8) -> Pin<'a, I2C, MODE> {
		Pin {
			driver,
			pin,
			_phantom: PhantomData,
		}
	}

	/// Get the polarity inversion of the current pin.
	pub fn polarity_inversion(&mut self) -> Result<Polarity, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let gppir = dev.read_reg(Register::GPPIR)?;
		if gppir & (1 << self.pin) == (1 << self.pin) {
			Ok(Polarity::High)
		} else {
			Ok(Polarity::Low)
		}
	}

	/// Set the polarity inversion of the current pin.
	pub fn set_polarity_inversion(&mut self, polarity: Polarity) -> Result<(), Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut gppir = dev.read_reg(Register::GPPIR)?;
		match polarity {
			Polarity::Low => gppir &= !(1 << self.pin),
			Polarity::High => gppir |= 1 << self.pin,
		}
		dev.write_reg(Register::GPPIR, gppir)?;
		Ok(())
	}
}

impl<'a, E, I2C> Pin<'a, I2C, Input>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	/// Configure the pin as an output pin.
	pub fn into_output_pin(self) -> Result<Pin<'a, I2C, Output>, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut gpdr = dev.read_reg(Register::GPDR)?;
		gpdr |= 1 << self.pin;
		dev.write_reg(Register::GPDR, gpdr)?;

		self.driver.pins.borrow_mut()[self.pin as usize] = PinMode::Output;
		Ok(Pin::new(self.driver, self.pin))
	}

	/// Configure the pin as an interrupt pin.
	pub fn into_interrupt_pin(self) -> Result<Pin<'a, I2C, Interrupt>, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut iegpior = dev.read_reg(Register::IEGPIOR)?;
		iegpior |= 1 << self.pin;
		dev.write_reg(Register::IEGPIOR, iegpior)?;

		self.driver.pins.borrow_mut()[self.pin as usize] = PinMode::Interrupt;
		Ok(Pin::new(self.driver, self.pin))
	}
}

impl<'a, E, I2C> InputPin for Pin<'a, I2C, Input>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	type Error = Error<E>;

	fn is_low(&self) -> Result<bool, Self::Error> {
		let mask = self.driver.device.borrow_mut().read_reg(Register::GPMR)?;
		Ok(mask & (1 << self.pin) == 0)
	}

	fn is_high(&self) -> Result<bool, Self::Error> {
		let mask = self.driver.device.borrow_mut().read_reg(Register::GPMR)?;
		Ok(mask & (1 << self.pin) == 1 << self.pin)
	}
}

impl<'a, E, I2C> Pin<'a, I2C, Output>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	/// Configure the pin as an input pin.
	pub fn into_input_pin(self) -> Result<Pin<'a, I2C, Input>, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut gpdr = dev.read_reg(Register::GPDR)?;
		gpdr &= !(1 << self.pin);
		dev.write_reg(Register::GPDR, gpdr)?;

		self.driver.pins.borrow_mut()[self.pin as usize] = PinMode::Input;
		Ok(Pin::new(self.driver, self.pin))
	}

	/// Configure the pin as an interrupt pin.
	pub fn into_interrupt_pin(self) -> Result<Pin<'a, I2C, Interrupt>, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut gpdr = dev.read_reg(Register::GPDR)?;
		gpdr &= !(1 << self.pin);
		dev.write_reg(Register::GPDR, gpdr)?;
		let mut iegpior = dev.read_reg(Register::IEGPIOR)?;
		iegpior |= 1 << self.pin;
		dev.write_reg(Register::IEGPIOR, iegpior)?;

		self.driver.pins.borrow_mut()[self.pin as usize] = PinMode::Interrupt;
		Ok(Pin::new(self.driver, self.pin))
	}
}

impl<'a, E, I2C> OutputPin for Pin<'a, I2C, Output>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	type Error = Error<E>;

	fn set_low(&mut self) -> Result<(), Self::Error> {
		let mask = self.driver.device.borrow_mut().read_reg(Register::GPSR)?;
		self.driver
			.device
			.borrow_mut()
			.write_reg(Register::GPSR, mask & !(1 << self.pin))
	}

	fn set_high(&mut self) -> Result<(), Self::Error> {
		let mask = self.driver.device.borrow_mut().read_reg(Register::GPSR)?;
		self.driver
			.device
			.borrow_mut()
			.write_reg(Register::GPSR, mask | (1 << self.pin))
	}
}

impl<'a, E, I2C> Pin<'a, I2C, Interrupt>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	/// Configure the pin as an input pin.
	pub fn into_input_pin(self) -> Result<Pin<'a, I2C, Input>, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut iegpior = dev.read_reg(Register::IEGPIOR)?;
		iegpior &= !(1 << self.pin);
		dev.write_reg(Register::IEGPIOR, iegpior)?;

		self.driver.pins.borrow_mut()[self.pin as usize] = PinMode::Input;
		Ok(Pin::new(self.driver, self.pin))
	}

	/// Configure the pin as an output pin.
	pub fn into_output_pin(self) -> Result<Pin<'a, I2C, Output>, Error<E>> {
		let mut dev = self.driver.device.borrow_mut();
		let mut gpdr = dev.read_reg(Register::GPDR)?;
		gpdr |= 1 << self.pin;
		dev.write_reg(Register::GPDR, gpdr)?;
		let mut iegpior = dev.read_reg(Register::IEGPIOR)?;
		iegpior &= !(1 << self.pin);
		dev.write_reg(Register::IEGPIOR, iegpior)?;

		self.driver.pins.borrow_mut()[self.pin as usize] = PinMode::Output;
		Ok(Pin::new(self.driver, self.pin))
	}
}

impl<'a, E, I2C> InputPin for Pin<'a, I2C, Interrupt>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	type Error = Error<E>;

	fn is_low(&self) -> Result<bool, Self::Error> {
		let mask = self.driver.device.borrow_mut().read_reg(Register::GPMR)?;
		Ok(mask & (1 << self.pin) == 0)
	}

	fn is_high(&self) -> Result<bool, Self::Error> {
		let mask = self.driver.device.borrow_mut().read_reg(Register::GPMR)?;
		Ok(mask & (1 << self.pin) == 1 << self.pin)
	}
}
