# Using `rtic-sync`

`rtic-sync` provides primitives that can be used for message passing and resource sharing in async context.

The important structs are:
* The `Arbiter`, which allows you to await access to a shared resource in async contexts without using `lock`.
* `Channel`, which allows you to communicate between tasks (both `async` and non-`async`).

For more information on these structs, see the [`rtic-sync` docs](https://docs.rs/rtic-sync)