# rs-fsrs

![](https://github.com/open-spaced-repetition/rs-fsrs/actions/workflows/check.yml/badge.svg)

A rust implementation of FSRS scheduler.

Install:

```toml
[dependencies]
rs-fsrs = { version = "1.2.1" }
```

Quickstart:

```rust
use chrono::Utc;
use rs_fsrs::{FSRS, Card, Rating};

fn main() {
    let fsrs = FSRS::default();
    let card = Card::new();

    let record_log = fsrs.repeat(card, Utc::now());
    for rating in Rating::iter() {
        let item = record_log[rating].to_owned();
        println!("{:?}", item.card);
        println!("{:?}", item.review_log);
    }
}
```

## Fractional Days

The library provides support for working with fractional days through the `FractionalDays` trait:

```rust
use chrono::Duration;
use rs_fsrs::FractionalDays;

fn main() {
    // Convert Duration to fractional days
    let duration = Duration::hours(36); // 1.5 days
    let days = duration.num_fractional_days(); // 1.5
    
    // Create Duration from fractional days
    let duration = Duration::fractional_days(2.5); // 2.5 days
}
```

This is useful when you need more precision than whole days, as chrono's `num_days()` method truncates fractional parts.

## Development

run

```sh
cargo fmt
cargo clippy -- -D clippy::nursery
cargo test --release
```

## Other implementation

[fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) contains a Rust API for training FSRS parameters, and for using them to schedule cards.

## Bindings

- [c/cpp](https://github.com/open-spaced-repetition/rs-fsrs-c)
- [python](https://github.com/open-spaced-repetition/rs-fsrs-python)
- [java](https://github.com/open-spaced-repetition/rs-fsrs-java)
- [nodejs](https://github.com/open-spaced-repetition/rs-fsrs-nodejs)

## LICENSE

MIT
