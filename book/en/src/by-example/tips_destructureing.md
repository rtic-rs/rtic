# Resource de-structure-ing

When having a task taking multiple resources it can help in readability to split
up the resource struct. Here are two examples on how this can be done:

``` rust
{{#include ../../../../examples/destructure.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example destructure
{{#include ../../../../ci/expected/destructure.run}}
```
