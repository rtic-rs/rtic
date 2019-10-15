# Real Time For the Masses

A concurrency framework for building real time systems.

## Features

- **Tasks** as the unit of concurrency [^1]. Tasks can be *event triggered*
  (fired in response to asynchronous stimuli) or spawned by the application on
  demand.

- **Message passing** between tasks. Specifically, messages can be passed to
  software tasks at spawn time.

- **A timer queue** [^2]. Software tasks can be scheduled to run at some time
  in the future. This feature can be used to implement periodic tasks.

- Support for prioritization of tasks and, thus, **preemptive multitasking**.

- **Efficient and data race free memory sharing** through fine grained *priority
  based* critical sections [^1].

- **Deadlock free execution** guaranteed at compile time. This is an stronger
  guarantee than what's provided by [the standard `Mutex`
  abstraction][std-mutex].

[std-mutex]: https://doc.rust-lang.org/std/sync/struct.Mutex.html

- **Minimal scheduling overhead**. The task scheduler has minimal software
  footprint; the hardware does the bulk of the scheduling.

- **Highly efficient memory usage**: All the tasks share a single call stack and
  there's no hard dependency on a dynamic memory allocator.

- **All Cortex-M devices are fully supported**.

- This task model is amenable to known WCET (Worst Case Execution Time) analysis
  and scheduling analysis techniques. (Though we haven't yet developed Rust
  friendly tooling for that.)

## Requirements

- Rust 1.36.0+

- Applications must be written using the 2018 edition.

## [User documentation](https://rtfm.rs)

## [API reference](https://rtfm.rs/api/rtfm/index.html)

## Chat
Join us and talk about RTFM in the [Matrix room][matrix-room].

[matrix-room]: https://matrix.to/#/#rtfm-rs:matrix.org

## Contributing
New features and big changes should go through the RFC process in the [dedicated RFC repository][rfcs].

[rfcs]: https://github.com/rtfm-rs/rfcs

## Acknowledgments

This crate is based on [the RTFM language][rtfm-lang] created by the Embedded
Systems group at [Luleå University of Technology][ltu], led by [Prof. Per
Lindgren][per].

[rtfm-lang]: http://www.rtfm-lang.org/
[ltu]: https://www.ltu.se/?l=en
[per]: https://www.ltu.se/staff/p/pln-1.11258?l=en

## References

[^1]: Eriksson, J., Häggström, F., Aittamaa, S., Kruglyak, A., & Lindgren, P.
   (2013, June). Real-time for the masses, step 1: Programming API and static
   priority SRP kernel primitives. In Industrial Embedded Systems (SIES), 2013
   8th IEEE International Symposium on (pp. 110-113). IEEE.

[^2]: Lindgren, P., Fresk, E., Lindner, M., Lindner, A., Pereira, D., & Pinho,
   L. M. (2016). Abstract timers and their implementation onto the arm cortex-m
   family of mcus. ACM SIGBED Review, 13(1), 48-53.

## License

All source code (including code snippets) is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0][L1])
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/licenses/MIT][L2])

[L1]: https://www.apache.org/licenses/LICENSE-2.0
[L2]: https://opensource.org/licenses/MIT

at your option.

The written prose contained within the book is licensed under the terms of the
Creative Commons CC-BY-SA v4.0 license ([LICENSE-CC-BY-SA](LICENSE-CC-BY-SA) or
[https://creativecommons.org/licenses/by-sa/4.0/legalcode][L3]).

[L3]: https://creativecommons.org/licenses/by-sa/4.0/legalcode

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
