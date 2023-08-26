# rs-fsrs
A rust implementation of FSRS.

Quickstart:
```rust
use chrono::Utc;
use fsrs::{FSRS, Card, Rating::Easy};

fn main() {
    let fsrs = FSRS::default();
    let card = Card::new();
    
    let scheduled_cards = fsrs.schedule(card, Utc::now());

    let updated_card = scheduled_cards.select_card(Easy);

    println!("{:?}", updated_card.log);
}
```

## LICENSE

MIT
