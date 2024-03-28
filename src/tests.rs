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
static WEIGHTS: [f32; 17] = [
    1.0171, 1.8296, 4.4145, 10.9355, 5.0965, 1.3322, 1.017, 0.0, 1.6243, 0.1369, 1.0321, 2.1866,
    0.0661, 0.336, 1.7766, 0.1693, 2.9244,
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
    let expected = [0, 4, 15, 49, 143, 379, 0, 0, 15, 37, 85, 184, 376];
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
    assert!((card.stability - 43.05542).abs() < f32::EPSILON * 100f32);
    assert_eq!(card.difficulty, 7.7609);
}
