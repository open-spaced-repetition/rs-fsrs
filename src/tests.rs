#[cfg(test)]
use {
    crate::{
        algo::FSRS,
        models::{Card, Rating, State},
        parameters::Parameters,
    },
    chrono::{DateTime, Duration, TimeZone, Utc},
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
static WEIGHTS: [f64; 19] = [
    0.4197, 1.1869, 3.0412, 15.2441, 7.1434, 0.6477, 1.0007, 0.0674, 1.6597, 0.1712, 1.1178,
    2.0225, 0.0904, 0.3025, 2.1214, 0.2498, 2.9466, 0.4891, 0.6468,
];

#[cfg(test)]
fn string_to_utc(date_string: &str) -> DateTime<Utc> {
    let datetime = DateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S %z %Z").unwrap();
    Utc.from_local_datetime(&datetime.naive_utc()).unwrap()
}

#[cfg(test)]
fn round_float(num: f64, precision: i32) -> f64 {
    let multiplier = 10.0_f64.powi(precision);
    (num * multiplier).round() / multiplier
}

#[test]
fn test_basic_scheduler_interval() {
    let params = Parameters {
        w: WEIGHTS,
        ..Default::default()
    };

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut interval_history = vec![];

    for rating in TEST_RATINGS.iter() {
        let next = fsrs.next(card, now, *rating);
        card = next.card;
        interval_history.push(card.scheduled_days);
        now = card.due;
    }
    let expected = [0, 4, 17, 62, 198, 563, 0, 0, 9, 27, 74, 190, 457];
    assert_eq!(interval_history, expected);
}

#[test]
fn test_basic_scheduler_state() {
    let params = Parameters {
        w: WEIGHTS,
        ..Default::default()
    };

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut state_list: Vec<State> = vec![];
    let mut scheduling_card = fsrs.repeat(card, now);

    for rating in TEST_RATINGS.iter() {
        card = scheduling_card[rating].card.clone();
        let rev_log = scheduling_card[rating].review_log.clone();
        state_list.push(rev_log.state);
        now = card.due;
        scheduling_card = fsrs.repeat(card, now);
    }
    use State::*;
    let expected = [
        New, Learning, Review, Review, Review, Review, Review, Relearning, Relearning, Review,
        Review, Review, Review,
    ];
    assert_eq!(state_list, expected);
}

#[test]
fn test_basic_scheduler_memo_state() {
    let params = Parameters {
        w: WEIGHTS,
        ..Default::default()
    };

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut scheduling_card = fsrs.repeat(card.clone(), now);
    let ratings = [
        Rating::Again,
        Rating::Good,
        Rating::Good,
        Rating::Good,
        Rating::Good,
        Rating::Good,
    ];
    let intervals = [0, 0, 1, 3, 8, 21];
    for (index, rating) in ratings.iter().enumerate() {
        card = scheduling_card[rating].card.clone();
        now = now + Duration::days(intervals[index] as i64);
        scheduling_card = fsrs.repeat(card.clone(), now);
    }

    card = scheduling_card.get(&Rating::Good).unwrap().to_owned().card;
    assert_eq!(round_float(card.stability, 4), 71.4554);
    assert_eq!(round_float(card.difficulty, 4), 5.0976);
}
