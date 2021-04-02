//! Platform-agnostic driver for the STMPE1600 I²C I/O expander.
//!
//! The STMPE1600 is a device that provide 16 GPIO pins, which are configurable through software as
//! either floating input or push-pull output. The pins can also be configured to use interrupts,
//! so that when they are triggered, they send a signal (of a configurable polarity) through the interrupt output pin.
//!
//! This driver is intended to work on any embedded platform, by using the [`embedded-hal`](https://crates.io/crates/embedded-hal) library.
//! This works by using the I²C traits of `embedded-hal`, which allows for a specific HAL (hardware abstraction layer) to provide its own interface
//! to allow using the specific implentation of I²C necessary to work on a specific device.
//!
//! # Driver construction
//! To construct the driver, you will need to use the [`Stmpe1600Builder`](struct.Stmpe1600Builder.html) struct.
//! For more information on what configuration options can be changed, view the `Stmpe1600Builder` documentation.
//! ```ignore
//! let i2c = /* construct something implementing embedded_hal::blocking::i2c::{Read, Write} */;
//! let stmpe1600 = Stmpe1600Builder::new(i2c).build()?;
//! ```
//!
//! # Accessing I/O
//! To access the I/O pins, call either [`Stmpe1600::pin_input`], [`Stmpe1600::pin_output`] or [`Stmpe1600::pin_interrupt`],
//! which will return a [`Pin`](struct.Pin.html) object.
//!
//! This type implements [`embedded_hal::digital::v2::InputPin`] or [`embedded_hal::digital::v2::OutputPin`] (depending on the pin's mode),
//! which means that they can also be passed to any function which takes these types as arguments;
//! this allows these pins to be passed transparently to platform-agnostic drivers easily and efficiently.
//!
//! # Examples
//! ## Connecting to a device with a custom I²C address
//! ```rust,ignore
//! use linux_embedded_hal::I2cdev;
//! use stmpe1600::Stmpe1600Builder;
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let stmpe1600 = Stmpe1600Builder::new(dev)
//! 	.address(0x43)
//! 	.build()
//! 	.expect("Could not initialise STMPE1600 driver");
//! ```
//!
//! ## Read and write I/O pins
//! ```rust,ignore
//! use embedded_hal::digital::v2::{InputPin, OutputPin};
//! use linux_embedded_hal::I2cdev;
//! use stmpe1600::{PinMode, Stmpe1600Builder};
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let stmpe1600 = Stmpe1600Builder::new(dev)
//! 	.build()
//! 	.expect("Could not initialise STMPE1600 driver");
//!
//! let input_pin = stmpe1600.pin_input(0);
//! let output_pin = stmpe1600.pin_output(1);
//!
//! if input_pin.is_high()? {
//! 	output_pin.set_high()?
//! } else {
//! 	output_pin.set_low()?;
//! }
//! ```

#![no_std]
#![warn(missing_docs)]

use core::cell::RefCell;
use core::fmt::Debug;
use embedded_hal::blocking::i2c::{Read, Write};

mod builder;
pub use builder::Stmpe1600Builder;
mod device;
use device::{Register, Stmpe1600Device};
mod pins;
use pins::modes;
pub use pins::Pin;

/// The default I²C address for the STMPE1600.
pub const DEFAULT_ADDRESS: u8 = 0x42;

/// The types that the pins on the STMPE1600 may be configured as.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum PinMode {
	Input,
	Output,
	Interrupt,
}

/// Input/Interrupt polarity.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polarity {
	Low,
	High,
}

/// The different types of errors that can occur while interacting with the STMPE1600.
#[derive(Debug)]
pub enum Error<E> {
	/// I²C bus error
	I2CError(E),
	/// Invalid device ID
	InvalidDeviceID,
}

/// A struct representing the STMPE1600 device driver.
#[derive(Debug)]
pub struct Stmpe1600<I2C> {
	device: RefCell<Stmpe1600Device<I2C>>,
	pins: RefCell<[PinMode; 16]>,
}

impl<I2C, E> Stmpe1600<I2C>
where
	I2C: Read<Error = E> + Write<Error = E>,
{
	/// Create a [`Pin`] which corresponds to the specified pin, configured in input mode.
	///
	/// If the specified pin is not already configured in input mode, the mode will be changed
	/// automatically.
	///
	/// This function will panic if `pin > 16`.
	pub fn pin_input(&mut self, pin: u8) -> Result<Pin<'_, I2C, modes::Input>, Error<E>> {
		assert!(pin < 16);
		let mode = self.pins.borrow()[pin as usize];
		match mode {
			PinMode::Input => Ok(Pin::new(self, pin)),
			PinMode::Output => Pin::<I2C, modes::Output>::new(self, pin).into_input_pin(),
			PinMode::Interrupt => Pin::<I2C, modes::Interrupt>::new(self, pin).into_input_pin(),
		}
	}

	/// Create a [`Pin`] which corresponds to the specified pin, configured in output mode.
	///
	/// If the specified pin is not already configured in output mode, the mode will be changed
	/// automatically.
	///
	/// This function will panic if `pin > 16`.
	pub fn pin_output(&mut self, pin: u8) -> Result<Pin<'_, I2C, modes::Output>, Error<E>> {
		assert!(pin < 16);
		let mode = self.pins.borrow()[pin as usize];
		match mode {
			PinMode::Input => Pin::<I2C, modes::Input>::new(self, pin).into_output_pin(),
			PinMode::Output => Ok(Pin::new(self, pin)),
			PinMode::Interrupt => Pin::<I2C, modes::Interrupt>::new(self, pin).into_output_pin(),
		}
	}

	/// Create a [`Pin`] which corresponds to the specified pin, configured in interrupt mode.
	///
	/// If the specified pin is not already configured in interrupt mode, the mode will be changed
	/// automatically.
	///
	/// This function will panic if `pin > 16`.
	pub fn pin_interrupt(&mut self, pin: u8) -> Result<Pin<'_, I2C, modes::Interrupt>, Error<E>> {
		assert!(pin < 16);
		let mode = self.pins.borrow()[pin as usize];
		match mode {
			PinMode::Input => Pin::<I2C, modes::Input>::new(self, pin).into_interrupt_pin(),
			PinMode::Output => Pin::<I2C, modes::Output>::new(self, pin).into_interrupt_pin(),
			PinMode::Interrupt => Ok(Pin::new(self, pin)),
		}
	}

	/// Gets the pending interrupts and returns them in an array.
	///
	/// This function clears any pending bits from the STMPE1600,
	/// and in doing so, stops triggering the interrupt output pin.
	pub fn get_interrupts(&self) -> Result<[bool; 16], Error<E>> {
		self.device.borrow_mut().get_interrupts()
	}
}
