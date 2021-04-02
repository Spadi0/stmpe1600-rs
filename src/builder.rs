use crate::device::{Register, Stmpe1600Device};
use crate::{Error, InterruptPolarity, PinMode, Stmpe1600, DEFAULT_ADDRESS};
use core::cell::RefCell;
use embedded_hal::blocking::i2c::{Read, Write};

/// A builder that allows for configuring all the various options available to edit on the STMPE1600.
pub struct Stmpe1600Builder<I2C> {
	i2c: I2C,
	pins: [PinMode; 16],
	address: u8,
	interrupt_polarity: Option<InterruptPolarity>,
}

impl<I2C, E> Stmpe1600Builder<I2C>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	/// Constructs a builder.
	pub fn new(i2c: I2C) -> Stmpe1600Builder<I2C> {
		Stmpe1600Builder {
			i2c,
			pins: [PinMode::Input; 16],
			address: DEFAULT_ADDRESS,
			interrupt_polarity: None,
		}
	}

	/// Sets the I²C address on which to attempt communication with the STMPE1600.
	pub fn address(mut self, address: u8) -> Stmpe1600Builder<I2C> {
		self.address = address;
		self
	}

	/// Enables interrupts, and sets the polarity of the interrupt output pin.
	pub fn interrupts(mut self, polarity: InterruptPolarity) -> Stmpe1600Builder<I2C> {
		self.interrupt_polarity = Some(polarity);
		self
	}

	/// Consumes the builder, and produces an [`Stmpe1600`](struct.Stmpe1600.html) struct.
	pub fn build(self) -> Result<Stmpe1600<I2C>, Error<E>> {
		let mut device = Stmpe1600Device::new(self.i2c, self.address)?;

		if let Some(polarity) = self.interrupt_polarity {
			let scb = device.read_reg8(Register::SystemControl)?;
			let polarity = match polarity {
				InterruptPolarity::Low => 0x00,
				InterruptPolarity::High => 0x01,
			};
			device.write_reg8(Register::SystemControl, scb | 0x04 | polarity)?;
		}

		Ok(Stmpe1600 {
			device: RefCell::new(device),
			pins: RefCell::new(self.pins),
		})
	}
}
