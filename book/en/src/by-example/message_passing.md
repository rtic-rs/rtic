# Message passing & capacity

Software tasks have support for message passing, this means that they can be spawned with an argument
as `foo::spawn(1)` which will run the task `foo` with the argument `1`. The number of arguments is not
limited and is exemplified in the following:

``` rust
{{#include ../../../../examples/message_passing.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example message_passing
{{#include ../../../../ci/expected/message_passing.run}}
```
