use stmpe1600::{DEFAULT_ADDRESS, PinMode, Register, Stmpe1600Builder};
use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

#[test]
fn basic_builder() {
	let expectations = [
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::ChipID as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::SystemControl as u8, 0x80]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPDR as u8, 0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::IEGPIOR as u8, 0x00, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let _stmpe1600 = Stmpe1600Builder::new(i2c).build().expect("Failed to initialise STMPE1600 driver");
}

#[test]
fn custom_address_builder() {
	let expectations = [
		I2cTransaction::write(0x43, vec![Register::ChipID as u8]),
		I2cTransaction::read(0x43, vec![0x00, 0x16]),
		I2cTransaction::write(0x43, vec![Register::SystemControl as u8, 0x80]),
		I2cTransaction::write(0x43, vec![Register::GPDR as u8, 0x00, 0x00]),
		I2cTransaction::write(0x43, vec![Register::IEGPIOR as u8, 0x00, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let _stmpe1600 = Stmpe1600Builder::new(i2c)
		.address(0x43)
		.build()
		.expect("Failed to initialise STMPE1600 driver with custom address");
}

#[test]
fn pin_mode_builder() {
	let expectations = [
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::ChipID as u8]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::SystemControl as u8, 0x80]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::GPDR as u8, 0x02, 0xFF]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![Register::IEGPIOR as u8, 0x04, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let _stmpe1600 = Stmpe1600Builder::new(i2c)
		.pin(1, PinMode::Output)
		.pin(2, PinMode::Interrupt)
		.pins(8..16, PinMode::Output)
		.build()
		.expect("Failed to initialise STMPE1600 driver with configured custom pinmodes");
}