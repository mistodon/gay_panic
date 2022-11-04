# gay_panic

A Rust panic handler, but make it gay.

[![CI](https://github.com/mistodon/gay_panic/actions/workflows/rust.yml/badge.svg)](https://github.com/mistodon/gay_panic/actions/workflows/rust.yml)
[![Docs.rs](https://docs.rs/gay_panic/badge.svg)](https://docs.rs/gay_panic/1.0.0/gay_panic/)
[![Crates.io](https://img.shields.io/crates/v/gay_panic.svg)](https://crates.io/crates/gay_panic)
<!-- [![codecov](https://codecov.io/github/mistodon/gay_panic/branch/main/graph/badge.svg?token=XN5QQCKX5Z)](https://codecov.io/github/mistodon/gay_panic) -->


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

![Rainbow backtrace](./pretty_gay_panic.png)
