# Contributing

## New features

New features should go through the [RFC process][rfcs] before a Pull Request is made to this repository.

[rfcs](https://github.com/rtic-rs/rfcs)

## Bugs

Report bugs by creating an issue in this repository.

## Pull Requests

Please make pull requests against the master branch.

Always use rebase instead of merge when bringing in changes from master to your feature branch.

## Writing documentation

Documentation improvements are always welcome. The source for the book is in `book/` and API documentation is generated from the source code.

## CI test preparation

To reduce risk of CI failing for your PR, please make sure that tests passes locally before submitting.

```shell
> cargo xtask --target all
```

Will execute `run` tests on your local `qemu` install. (You may also pass a single target `--target thumbv6m-none-eabi/thumbv7m-none-eabi` during your development). These test are quite time consuming as they compile and run all `examples`.

If you have added further tests, you need to add the expected output in the `ci/expected` folder.

```shell
>  cargo run --example <NAME> --target thumbv7m-none-eabi > ci/expected/<NAME>.run
```

Internal fail tests can be locally run:

```shell
> cargo test --tests
```

If you have added fail tests or changed the expected behavior, the expected output needs to be updated (corresponding `.stderr` files). Inspect the error output, when sure that `ACTUAL OUTPUT` is correct you can re-run the test as:

```shell
> TRYBUILD=overwrite cargo test --tests
```

This will update the expected output to match the `ACTUAL OUTPUT`. Please check that the updated files are indeed correct as to avoid regressions.

Once all tests pass you are ready to make a PR.
