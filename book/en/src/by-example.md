# RTFM by example

This part of the book introduces the Real Time For the Masses (RTFM) framework
to new users by walking them through examples of increasing complexity.

All examples in this part of the book can be found in the GitHub [repository] of
the project, and most of the examples can be run on QEMU so no special hardware
is required to follow along.

[repository]: https://github.com/japaric/cortex-m-rtfm

To run the examples on your laptop / PC you'll need the `qemu-system-arm`
program. Check [the embedded Rust book] for instructions on how to set up an
embedded development environment that includes QEMU.

[the embedded Rust book]: https://rust-embedded.github.io/book/intro/install.html

## Real World Examples

The following are examples of RTFM being used in real world projects.

### RTFM V0.4.2

- [etrombly/sandbox](https://github.com/etrombly/sandbox/tree/41d423bcdd0d8e42fd46b79771400a8ca349af55). A hardware zen garden that draws patterns in sand. Patterns are sent over serial using G-code.
