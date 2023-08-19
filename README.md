# rs-fsrs
A rust implementation of FSRS.

Quickstart:
```rust
use chrono::Utc;
use fsrs::FSRS;
use fsrs::models::{Card, Rating::Easy};


fn main() {
    let mut fsrs = FSRS::default();

    let mut card = Card::new();
    fsrs.schedule(&mut card, Utc::now());

    card = fsrs.select_card(Easy);

    println!("{}", card.scheduled_days);
}
```

## LICENSE

MIT
