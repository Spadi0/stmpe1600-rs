use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use stmpe1600::{PinMode, Register, Stmpe1600Builder, DEFAULT_ADDRESS};

#[test]
fn read_pin() {
	let expectations = [
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::ChipID as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::SystemControl as u8, 0x80]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPDR as u8, 0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::IEGPIOR as u8, 0x00, 0x00]),
		// read pin operation
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPMR as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x01, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let stmpe1600 = Stmpe1600Builder::new(i2c)
		.build()
		.expect("Failed to initialise STMPE1600 driver");

	let input_pin = stmpe1600.pin(0);
	let is_high = input_pin.is_high().unwrap();
	assert!(is_high, "Input pin in is LOW");
}

#[test]
fn write_pin() {
	let expectations = [
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::ChipID as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::SystemControl as u8, 0x80]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPDR as u8, 0x01, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::IEGPIOR as u8, 0x00, 0x00]),
		// write pin operation
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPSR as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPSR as u8, 0x01, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let stmpe1600 = Stmpe1600Builder::new(i2c)
		.pin(0, PinMode::Output)
		.build()
		.expect("Failed to initialise STMPE1600 driver");

	let mut output_pin = stmpe1600.pin(0);
	output_pin.set_high().unwrap();
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: IncorrectPinMode")]
fn read_from_output_pin() {
	let expectations = [
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::ChipID as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::SystemControl as u8, 0x80]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPDR as u8, 0x01, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::IEGPIOR as u8, 0x00, 0x00]),
		// write pin operation
		I2cTransaction::read(DEFAULT_ADDRESS, vec![Register::GPSR as u8]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPSR as u8, 0x01, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let stmpe1600 = Stmpe1600Builder::new(i2c)
		.pin(0, PinMode::Output)
		.build()
		.expect("Failed to initialise STMPE1600 driver");

	let output_pin = stmpe1600.pin(0);
	let _is_high = output_pin.is_high().unwrap();
}
