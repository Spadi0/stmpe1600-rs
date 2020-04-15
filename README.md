# STMPE1600 I/O Expander Rust Driver

This is a platform-agnostic Rust driver for the [STMPE1600 I/O expander](https://www.st.com/en/interfaces-and-transceivers/stmpe1600.html).

This driver can:
- Setup the pins as input, output or interrupt pins.
- Get the state of all the pins at once.
- Set the state of all the pins at once.
- Have an interrupt callback which is triggered on interrupt.

## Interrupts

The STMPE1600 handles interrupts by triggering an interrupt output pin when it detects an interrupt on any of its configured interrupt pins. The active LOW interrupt pin needs to be handled by the microcontroller itself, and then call the [`handle_interrupt`] function to check the STMPE1600 for any pending interrupts, execute the relevent callbacks and clear the pending bits.

## Usage

See the [examples](examples/) directory for usage examples.

```rust
use linux_embedded_hal::I2cdev;
use stmpe1600::{PinFlag, Stmpe1600};

fn main() {
	let i2c = I2cdev::new("/dev/i2c-1").unwrap();
	let expander = Stmpe1600::new(i2c).unwrap();
	expander.setup_output_pins(PinFlag::P0).unwrap();
	expander.set_state(PinFlag::P0).unwrap();

	println!("Pin Status: {:?}" expander.get_state().unwrap())
}
```

## To-Do
- [ ] Add interrupt polarity
- [ ] Create a way to access each of the pins individually
- [ ] Implement [`embedded_hal`] InputPin and OutputPin traits