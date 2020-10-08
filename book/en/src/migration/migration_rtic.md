# Migrating from RTFM to RTIC

This section covers how to upgrade an application written against RTFM v0.5.x to
the same version of RTIC. This applies since the renaming of the framework as per [RFC #33].

**Note:** There are no code differences between RTFM v0.5.3 and RTIC v0.5.3, it is purely a name
change.

[RFC #33]: https://github.com/rtic-rs/rfcs/pull/33

## `Cargo.toml`

First, the `cortex-m-rtfm` dependency needs to be updated to
`cortex-m-rtic`.

``` toml
[dependencies]
# change this
cortex-m-rtfm = "0.5.3"

# into this
cortex-m-rtic = "0.5.3"
```

## Code changes

The only code change that needs to be made is that any reference to `rtfm` before now need to point
to `rtic` as follows:

``` rust
//
// Change this
//

#[rtfm::app(/* .. */, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    // ...

};

//
// Into this
//

#[rtic::app(/* .. */, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // ...

};
```
