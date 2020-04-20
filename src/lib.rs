//! Platform-agnostic driver for the STMPE1600 I²C I/O expander.
//!
//! The STMPE1600 is a device that provide 16 GPIO pins, which are configurable through software as
//! either floating input or push-pull output. The pins can also be configured to use interrupts,
//! so that when they are triggered, they send a signal (of a configurable polarity) through the interrupt output pin.
//! 
//! This driver is intended to work on any embedded platform, by using the [`embedded-hal`](https://crates.io/crates/embedded-hal) library.
//! This works by using the I²C traits of embedded-hal, which allows for a specific HAL (hardware abstraction layer) to provide its own interface
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
//! TODO: implement me
//! 
//! # Examples
//! ## Connecting to a device with a custom I²C address
//! ```rust,no_run
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
//! ## Setting all the pins to output mode
//! ```rust,no_run
//! use linux_embedded_hal::I2cdev;
//! use stmpe1600::{PinMode, Stmpe1600Builder};
//! 
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let stmpe1600 = Stmpe1600Builder::new(dev)
//! 	.pins(0..16, PinMode::Output)
//! 	.build()
//! 	.expect("Could not initialise STMPE1600 driver");
//! ```

#![no_std]
#![warn(missing_docs)]

use core::fmt::Debug;
use core::cell::RefCell;
use embedded_hal::blocking::i2c::{Read, Write};

mod builder;
pub use builder::Stmpe1600Builder;
mod device;
use device::{Register, Stmpe1600Device};

/// The default I²C address for the STMPE1600.
pub const DEFAULT_ADDRESS: u8 = 0x42;

/// A struct representing the STMPE1600 device driver.
#[derive(Debug)]
pub struct Stmpe1600<I2C> {
	device: RefCell<Stmpe1600Device<I2C>>
}

/// The types that the pins on the STMPE1600 may be configured as.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum PinMode {
	Input,
	Output,
	Interrupt
}

/// Tells the STMPE1600 what value the interrupt output pin should be set to, when an interrupt is triggered.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum InterruptPolarity {
	Low,
	High,
}

/// The different types of errors that can occur while interacting with the STMPE1600.
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
	/// Gets the pending interrupts and returns them in an array.
	/// 
	/// This function clears any pending bits from the STMPE1600,
	/// and in doing so, stops triggering the interrupt output pin.
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