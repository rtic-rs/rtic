# Contributing

## New features

New features should go through the [RFC process][rfcs] before creating a Pull Request to this repository.

[rfcs]: https://github.com/rtic-rs/rfcs

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
> cargo xtask example-check
```

### Run examples/tests on QEMU device

```shell
> cargo xtask qemu
```

Will execute examples on your local `qemu` install.

#### Adding examples/tests to xtask

If you have added further tests, you need to add the expected output in the `ci/expected` folder.

```shell
>  cargo xtask qemu --overwrite-expected
```

### Internal tests

Run internal fail tests locally with:

```shell
> cargo xtask test
```

#### Adding tests to internal tests

If you have added fail tests or changed the expected behavior, the expected output needs to be updated (corresponding `.stderr` files).
Inspect the error output, when sure that `ACTUAL OUTPUT` is correct you can re-run the test as:

```shell
> TRYBUILD=overwrite cargo xtask test
```

This will update the expected output to match the `ACTUAL OUTPUT`.
Please check that the updated files are indeed correct to avoid regressions.
