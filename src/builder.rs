use core::cell::RefCell;
use core::fmt::Debug;
use core::iter::IntoIterator;

use embedded_hal::blocking::i2c::{Read, Write};
use crate::{DEFAULT_ADDRESS, Error, InterruptPolarity, PinMode, Stmpe1600};
use crate::device::{Register, Stmpe1600Device};

pub struct Stmpe1600Builder<I2C> {
	i2c: I2C,
	pins: [PinMode; 16],
	address: u8,
	use_interrupts: bool,
	interrupt_polarity: InterruptPolarity,
}

impl<I2C, E> Stmpe1600Builder<I2C>
	where I2C: Read<Error = E> + Write<Error = E>, E: Debug
{
	pub fn new(i2c: I2C) -> Stmpe1600Builder<I2C> {
		Stmpe1600Builder {
			i2c,
			pins: [PinMode::Input; 16],
			address: DEFAULT_ADDRESS,
			use_interrupts: false,
			interrupt_polarity: InterruptPolarity::Low,
		}
	}

	pub fn address(&mut self, address: u8) -> &mut Stmpe1600Builder<I2C> {
		self.address = address;
		self
	}

	pub fn pin(&mut self, pin: u8, mode: PinMode) -> &mut Stmpe1600Builder<I2C> {
		self.set_pin(pin, mode);
		self
	}

	pub fn pins<I>(&mut self, pins: I, mode: PinMode) -> &mut Stmpe1600Builder<I2C>
		where I: IntoIterator<Item = u8>
	{
		for pin in pins {
			self.set_pin(pin, mode);
		}
		self
	}

	pub fn interrupts(&mut self, polarity: InterruptPolarity) -> &mut Stmpe1600Builder<I2C> {
		self.use_interrupts = true;
		self.interrupt_polarity = polarity;
		self
	}

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
			let scb = device.read_reg(Register::SystemControl)?;
			let polarity = match self.interrupt_polarity {
				InterruptPolarity::Low => 0x00,
				InterruptPolarity::High => 0x01,
			};
			device.write_reg(Register::SystemControl, scb | 0x04 | polarity)?;
		}

		Ok(Stmpe1600 { device: RefCell::new(device) })
	}

	fn set_pin(&mut self, pin: u8, mode: PinMode) {
		assert!(pin < 16);
		self.pins[pin as usize] = mode;
	}
}

// fn main() {
// 	let stmpe1600 =
// 		Stmpe1600::builder(i2c)
// 			.address(0x42)
// 			.pin(8, PinMode::Input)
// 			.pins(9..16, PinMode::Interrupt)
// 			.enable_interrupts()
// 			.build().unwrap();
// }

// fn EXTI9_5() {
// 	stmpe1600.get_interrupts();
// 	[false, false, false, true, false, ...]
// 	if interrupts[8] {

// 	}
// }