# rs-fsrs

![](https://github.com/open-spaced-repetition/rs-fsrs/actions/workflows/check.yml/badge.svg)

A rust implementation of FSRS scheduler.

Install:

```toml
[dependencies]
fsrs = { git = "https://github.com/open-spaced-repetition/rs-fsrs" }
chrono = { version = "0.4.23", features = ["serde"] }
```

Quickstart:

```rust
use chrono::Utc;
use fsrs::{FSRS, Card, Rating};

fn main() {
    let fsrs = FSRS::default();
    let card = Card::new();

    let scheduleing_card = fsrs.repeat(card, Utc::now());
    for rating in Rating::iter() {
        let item = scheduleing_card.get(rating).unwrap().to_owned();
        println!("{:?}", item.card);
        println!("{:?}", item.review_log);
    }
}
```

## Development

run

```sh
cargo fmt
cargo clippy -- -Dwarnings
cargo clippy -- -D clippy::nursery
cargo test --release
```

## Other implementation

[fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) contains a Rust API for training FSRS parameters, and for using them to schedule cards.

## LICENSE

MIT
