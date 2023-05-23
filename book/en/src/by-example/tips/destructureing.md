# Resource de-structure-ing

Destructuring task resources might help readability if a task takes multiple
resources. Here are two examples on how to split up the resource struct:

``` rust,noplayground
{{#include ../../../../../rtic/examples/destructure.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example destructure
```
``` console
{{#include ../../../../../rtic/ci/expected/destructure.run}}
```
