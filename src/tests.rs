#[cfg(test)]
use {
    crate::{
        algo::FSRS,
        models::{Card, Parameters, Rating, State},
    },
    chrono::{DateTime, Days, TimeZone, Utc},
};

#[cfg(test)]
static TEST_RATINGS: [Rating; 13] = [
    Rating::Good,
    Rating::Good,
    Rating::Good,
    Rating::Good,
    Rating::Good,
    Rating::Good,
    Rating::Again,
    Rating::Again,
    Rating::Good,
    Rating::Good,
    Rating::Good,
    Rating::Good,
    Rating::Good,
];

#[cfg(test)]
static WEIGHTS: [f32; 19] = [
    0.4197, 1.1869, 3.0412, 15.2441, 7.1434, 0.6477, 1.0007, 0.0674, 1.6597, 0.1712, 1.1178,
    2.0225, 0.0904, 0.3025, 2.1214, 0.2498, 2.9466, 0.4891, 0.6468,
];

#[cfg(test)]
fn string_to_utc(date_string: &str) -> DateTime<Utc> {
    let datetime = DateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S %z %Z").unwrap();
    Utc.from_local_datetime(&datetime.naive_utc()).unwrap()
}

#[test]
fn test_interval() {
    let params = Parameters {
        w: WEIGHTS,
        ..Default::default()
    };

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut interval_history = vec![];

    for rating in TEST_RATINGS.iter() {
        let scheduled_cards = fsrs.schedule(card, now);
        card = scheduled_cards.select_card(*rating);

        interval_history.push(card.scheduled_days);
        now = card.due;
    }
    let expected = [0, 4, 17, 62, 198, 563, 0, 0, 9, 27, 74, 190, 457];
    assert_eq!(interval_history, expected);
}

#[test]
fn test_state() {
    let params = Parameters {
        w: WEIGHTS,
        ..Default::default()
    };

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut state_history = vec![];

    for rating in TEST_RATINGS.iter() {
        state_history.push(card.state);
        let scheduled_cards = fsrs.schedule(card, now);

        card = scheduled_cards.select_card(*rating);
        now = card.due;
    }
    use State::*;
    let expected = [
        New, Learning, Review, Review, Review, Review, Review, Relearning, Relearning, Review,
        Review, Review, Review,
    ];
    assert_eq!(state_history, expected);
}

#[test]
fn test_memo_state() {
    let params = Parameters {
        w: WEIGHTS,
        ..Default::default()
    };

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");

    let ratings = [
        Rating::Again,
        Rating::Good,
        Rating::Good,
        Rating::Good,
        Rating::Good,
        Rating::Good,
    ];
    let intervals = [0, 0, 1, 3, 8, 21];
    let scheduled_cards = ratings.iter().zip(intervals.iter()).fold(
        fsrs.schedule(card.clone(), now),
        |scheduled_cards, (rating, interval)| {
            card = scheduled_cards.select_card(*rating);
            now = now.checked_add_days(Days::new(*interval)).unwrap();
            fsrs.schedule(card.clone(), now)
        },
    );
    card = scheduled_cards.select_card(Rating::Good);
    assert_eq!(card.stability, 71.4554);
    // card.difficulty = 5.0976353
    assert!((card.difficulty - 5.0976).abs() < f32::EPSILON * 1000f32)
}
