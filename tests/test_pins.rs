use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use stmpe1600::{Polarity, Stmpe1600Builder, DEFAULT_ADDRESS};

#[test]
fn read_pin() {
	let i2c = I2cMock::new(&[
		// Check device ID.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x00]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		// Software reset.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x03, 0x80]),
		// Get pin 0 state.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x10]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x01, 0x00]),
	]);

	let mut stmpe1600 = Stmpe1600Builder::new(i2c).build().unwrap();
	let input_pin = stmpe1600.pin_input(0).unwrap();
	assert!(input_pin.is_high().unwrap(), "Input pin in is LOW");
}

#[test]
fn write_pin() {
	let i2c = I2cMock::new(&[
		// Check device ID.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x00]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		// Software reset.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x03, 0x80]),
		// Set pin 0 as an output pin.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x14]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x14, 0x01, 0x00]),
		// Set pin 0 as HIGH.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x12]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x12, 0x01, 0x00]),
	]);

	let mut stmpe1600 = Stmpe1600Builder::new(i2c).build().unwrap();
	let mut output_pin = stmpe1600.pin_output(0).unwrap();
	output_pin.set_high().unwrap();
}

#[test]
fn polarity_inversion() {
	let i2c = I2cMock::new(&[
		// Check device ID.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x00]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		// Software reset.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x03, 0x80]),
		// Get pin 0 polarity inversion.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x16]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x00]),
		// Set pin 0 polarity inversion to HIGH.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x16]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x16, 0x01, 0x00]),
		// Get pin 0 polarity inversion.
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x16]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x01, 0x00]),
	]);

	let mut stmpe1600 = Stmpe1600Builder::new(i2c).build().unwrap();
	let mut pin = stmpe1600.pin_input(0).unwrap();
	assert_eq!(pin.polarity_inversion().unwrap(), Polarity::Low);
	pin.set_polarity_inversion(Polarity::High).unwrap();
	assert_eq!(pin.polarity_inversion().unwrap(), Polarity::High);
}
