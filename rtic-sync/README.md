To run the repro test:

```
RUSTFLAGS="--cfg loom" cargo test --features loom,testing --release --lib
```