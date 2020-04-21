# STMPE1600 I/O Expander Rust Driver

[![Crate](https://img.shields.io/crates/v/stmpe1600.svg)](https://crates.io/crates/stmpe1600)
[![Docs](https://docs.rs/stmpe1600/badge.svg)](https://docs.rs/stmpe1600)

This is a platform-agnostic Rust driver for the [STMPE1600 I/O expander](https://www.st.com/en/interfaces-and-transceivers/stmpe1600.html).

This driver can:
- Setup the pins as input, output or interrupt pins.
- Read/write to a specific pin.
- Enable interrupt capability.
- Set the interrupt output polarity.

## Interrupts

The STMPE1600 handles interrupts by triggering an interrupt output pin when it detects an interrupt on any of its configured interrupt pins.
The polarity of the interrupt output pin can be configured to be HIGH or LOW, and when the interrupt is triggered, the microcontroller can
get any pending interrupts by calling `get_interrupts`, which will also clear the pending interrupts on the STMPE1600 itself.

## Usage
See [docs](https://docs.rs/stmpe1600).

## To-Do
- [X] Add interrupt polarity
- [X] Create a way to access each of the pins individually
- [X] Implement [embedded_hal](https://github.com/rust-embedded/embedded-hal) InputPin and OutputPin traits

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.