To run the repro test:

```
LOOM_CHECKPOINT_FILE=checkpoint.json LOOM_CHECKPOINT_INTERVAL=1 RUSTFLAGS="--cfg loom" cargo test --features loom,testing --lib
```

There's also a poisoning error in the `loom_cs` implementation. I believe it is not relevant, but merely caused by the failure: the panic in the critical section
causes the lock to be poisoned, but it is needed one more time when calling `Drop` for `Sender`.