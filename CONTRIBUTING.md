# Contributing

## New features

New features should go through the [RFC process][rfcs] before creating a Pull Request to this repository.

[rfcs](https://github.com/rtic-rs/rfcs)

## Bugs

Report bugs by creating an issue in this repository.

## Pull Requests (PRs)

Please make pull requests against the master branch.

Always use rebase instead of merge when bringing in changes from master to your feature branch.

## Writing documentation

Documentation improvements are always welcome.
The source for the book is in `book/` and API documentation is generated from the source code.

## CI test preparation

Continuous Integration (CI) tests are run against all pull requests.

Please make sure that tests passes locally before submitting.

### Cargo format

```shell
> cargo fmt
```

### Example check

```shell
> cargo check --examples --target thumbv7m-none-eabi
```

and/or

```shell
> cargo check --examples --target thumbv6m-none-eabi
```

### Run tests with xtask

```shell
> cargo xtask --target all
```

Will execute `run` tests on your local `qemu` install.
(You may also pass a single target `--target thumbv6m-none-eabi/thumbv7m-none-eabi` during development).

#### Adding tests to xtask

If you have added further tests, you need to add the expected output in the `ci/expected` folder.

```shell
>  cargo run --example <NAME> --target thumbv7m-none-eabi > ci/expected/<NAME>.run
```

### Internal tests

Run internal fail tests locally with:

```shell
> cargo test --tests
```

#### Adding tests to internal tests

If you have added fail tests or changed the expected behavior, the expected output needs to be updated (corresponding `.stderr` files).
Inspect the error output, when sure that `ACTUAL OUTPUT` is correct you can re-run the test as:

```shell
> TRYBUILD=overwrite cargo test --tests
```

This will update the expected output to match the `ACTUAL OUTPUT`.
Please check that the updated files are indeed correct to avoid regressions.
