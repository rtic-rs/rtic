# The minimal app

This is the smallest possible RTIC application:

``` rust,noplayground
{{#include ../../../../rtic/examples/smallest.rs}}
```

RTIC is designed with resource efficiency in mind. RTIC itself does not rely on any dynamic memory allocation, thus RAM requirement is dependent only on the application. The flash memory footprint is below 1kB including the interrupt vector table.

For a minimal example you can expect something like:
``` console
$ cargo size --example smallest --target thumbv7m-none-eabi --release
```

``` console
Finished release [optimized] target(s) in 0.07s
   text    data     bss     dec     hex filename
    924       0       0     924     39c smallest
```

<!-- ---

Technically, RTIC will generate a statically allocated future for each *software* task (holding the execution context, including the `Context` struct and stack allocated variables). Futures associated to the same static priority will share an asynchronous stack during execution.  -->
