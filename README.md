Parallel Job execution in Rust
------------------------------

Take a slice, chop it up, and have workers process each portion of it.

By ensuring all tasks end before the call inthe parent returns, and that the
parent outlives all children, we can get concurrency on non-`Share` primitives,
which initially means &[T] and a shared non-mutable closure state.
