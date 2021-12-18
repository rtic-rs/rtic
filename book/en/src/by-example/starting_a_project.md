# Starting a new project

A recommendation when starting a RTIC project from scratch is to follow RTIC's [`defmt-app-template`].

[`defmt-app-template`]: https://github.com/rtic-rs/defmt-app-template

This will give you an RTIC application with support for RTT logging with [`defmt`] and stack overflow
protection using [`flip-link`]. There are also a multitude of examples available provided by the community:

- [`rtic-examples`] - Multiple projects
- [https://github.com/kalkyl/f411-rtic](https://github.com/kalkyl/f411-rtic)
- ... More to come

[`defmt`]: https://github.com/knurling-rs/defmt/
[`flip-link`]: https://github.com/knurling-rs/flip-link/
[`rtic-examples`]: https://github.com/rtic-rs/rtic-examples
