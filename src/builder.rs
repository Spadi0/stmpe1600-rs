use core::cell::RefCell;
use core::fmt::Debug;
use core::iter::IntoIterator;

use crate::device::{Register, Stmpe1600Device};
use crate::{Error, InterruptPolarity, PinMode, Stmpe1600, DEFAULT_ADDRESS};
use embedded_hal::blocking::i2c::{Read, Write};

/// A builder that allows for configuring all the various options available to edit on the STMPE1600.
pub struct Stmpe1600Builder<I2C> {
	i2c: I2C,
	pins: [PinMode; 16],
	address: u8,
	use_interrupts: bool,
	interrupt_polarity: InterruptPolarity,
}

impl<I2C, E> Stmpe1600Builder<I2C>
where
	I2C: Read<Error = E> + Write<Error = E>,
	E: Debug,
{
	/// Constructs a builder.
	pub fn new(i2c: I2C) -> Stmpe1600Builder<I2C> {
		Stmpe1600Builder {
			i2c,
			pins: [PinMode::Input; 16],
			address: DEFAULT_ADDRESS,
			use_interrupts: false,
			interrupt_polarity: InterruptPolarity::Low,
		}
	}

	/// Sets the I²C address on which to attempt communication with the STMPE1600.
	pub fn address(mut self, address: u8) -> Stmpe1600Builder<I2C> {
		self.address = address;
		self
	}

	/// Sets the mode of the specified pin. Defaults to [`Input`](enum.PinMode.html#variant.Input).
	///
	/// To edit multiple pins at once, see [`pins`](#method.pins).
	pub fn pin(mut self, pin: u8, mode: PinMode) -> Stmpe1600Builder<I2C> {
		self.set_pin(pin, mode);
		self
	}

	/// Sets the mode of multiple pins at once. Defaults to [`Input`](enum.PinMode.html#variant.Input).
	///
	/// To edit a single pin, see [`pin`](#method.pin).
	pub fn pins<I>(mut self, pins: I, mode: PinMode) -> Stmpe1600Builder<I2C>
	where
		I: IntoIterator<Item = u8>,
	{
		for pin in pins {
			self.set_pin(pin, mode);
		}
		self
	}

	/// Enables interrupts, and sets the polarity of the interrupt output pin.
	pub fn interrupts(mut self, polarity: InterruptPolarity) -> Stmpe1600Builder<I2C> {
		self.use_interrupts = true;
		self.interrupt_polarity = polarity;
		self
	}

	/// Consumes the builder, and produces an [`Stmpe1600`](struct.Stmpe1600.html) struct.
	pub fn build(self) -> Result<Stmpe1600<I2C>, Error<E>> {
		let mut device = Stmpe1600Device::new(self.i2c, self.address)?;

		let mut gpdr = 0;
		let mut iegpior = 0;
		for (i, pin) in self.pins.iter().enumerate() {
			match pin {
				PinMode::Input => continue,
				PinMode::Output => gpdr |= 1 << i,
				PinMode::Interrupt => iegpior |= 1 << i,
			}
		}
		device.write_reg(Register::GPDR, gpdr)?;
		device.write_reg(Register::IEGPIOR, iegpior)?;

		if self.use_interrupts {
			let scb = device.read_reg8(Register::SystemControl)?;
			let polarity = match self.interrupt_polarity {
				InterruptPolarity::Low => 0x00,
				InterruptPolarity::High => 0x01,
			};
			device.write_reg8(Register::SystemControl, scb | 0x04 | polarity)?;
			// Clear pin input register
			device.read_reg(Register::GPMR)?;
		}

		Ok(Stmpe1600 {
			device: RefCell::new(device),
			pins: self.pins,
		})
	}

	fn set_pin(&mut self, pin: u8, mode: PinMode) {
		assert!(pin < 16);
		self.pins[pin as usize] = mode;
	}
}
