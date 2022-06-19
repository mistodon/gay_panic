# gay_panic

A Rust panic handler, but make it gay.

[![Crates.io](https://img.shields.io/crates/v/gay_panic.svg)](https://crates.io/crates/gay_panic)
[![Docs.rs](https://docs.rs/gay_panic/badge.svg)](https://docs.rs/gay_panic/0.1.0/gay_panic/)


A panic handler that shows pretty backtraces:

```rust
fn main() {
    use gay_panic::Config;

    gay_panic::init_with(Config {
        call_previous_hook: false,
        force_capture_backtrace: true,
    });
}
```

![./pretty_gay_panic.png]

**Note:** This crate currently requires nightly Rust to compile.
