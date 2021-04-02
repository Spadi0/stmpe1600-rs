use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use stmpe1600::{Stmpe1600Builder, DEFAULT_ADDRESS};

#[test]
fn basic_builder() {
	let expectations = [
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x00]),
		I2cTransaction::read(DEFAULT_ADDRESS, vec![0x00, 0x16]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x03, 0x80]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x14, 0x00, 0x00]),
		I2cTransaction::write(DEFAULT_ADDRESS, vec![0x08, 0x00, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let _stmpe1600 = Stmpe1600Builder::new(i2c)
		.build()
		.expect("Failed to initialise STMPE1600 driver");
}

#[test]
fn custom_address_builder() {
	let expectations = [
		I2cTransaction::write(0x43, vec![0x00]),
		I2cTransaction::read(0x43, vec![0x00, 0x16]),
		I2cTransaction::write(0x43, vec![0x03, 0x80]),
		I2cTransaction::write(0x43, vec![0x14, 0x00, 0x00]),
		I2cTransaction::write(0x43, vec![0x08, 0x00, 0x00]),
	];
	let i2c = I2cMock::new(&expectations);
	let _stmpe1600 = Stmpe1600Builder::new(i2c)
		.address(0x43)
		.build()
		.expect("Failed to initialise STMPE1600 driver with custom address");
}
