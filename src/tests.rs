#[cfg(test)]
use {
    crate::models::ReviewLog,
    crate::models::State,
    crate::{
        algo::FSRS,
        models::{Card, Parameters, Rating},
    },
    chrono::{DateTime, TimeZone, Utc},
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
    1.14, 1.01, 5.44, 14.67, 5.3024, 1.5662, 1.2503, 0.0028, 1.5489, 0.1763, 0.9953, 2.7473,
    0.0179, 0.3105, 0.3976, 0.0, 2.0902,
];

#[cfg(test)]
fn string_to_utc(date_string: &str) -> DateTime<Utc> {
    let datetime: DateTime<chrono::FixedOffset> =
        DateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S %z %Z").unwrap();
    Utc.from_local_datetime(&datetime.naive_utc()).unwrap()
}

#[test]
fn test_interval() {
    let mut params = Parameters::default();
    params.w = WEIGHTS;

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut interval_history: Vec<i64> = Vec::new();

    for rating in TEST_RATINGS.iter() {
        let scheduled_cards = fsrs.schedule(card, now);
        card = scheduled_cards.select_card(*rating);

        interval_history.push(card.scheduled_days);
        now = card.due;
    }
    let expected: Vec<i64> = vec![0, 5, 16, 43, 106, 236, 0, 0, 12, 25, 47, 85, 147];
    assert_eq!(interval_history, expected);
}

#[test]
fn test_state() {
    let mut params = Parameters::default();
    params.w = WEIGHTS;

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut state_history: Vec<State> = Vec::new();

    for rating in TEST_RATINGS.iter() {
        state_history.push(card.state);
        let scheduled_cards = fsrs.schedule(card, now);

        card = scheduled_cards.select_card(*rating);
        now = card.due;
    }
    let expected: Vec<State> = vec![
        State::New,
        State::Learning,
        State::Review,
        State::Review,
        State::Review,
        State::Review,
        State::Review,
        State::Relearning,
        State::Relearning,
        State::Review,
        State::Review,
        State::Review,
        State::Review,
    ];
    assert_eq!(state_history, expected);
}

#[test]
fn test_logs() {
    let mut log_history: Vec<ReviewLog> = Vec::new();
    let mut params = Parameters::default();
    params.w = WEIGHTS;

    let fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = string_to_utc("2022-11-29 12:30:00 +0000 UTC");
    let mut state_history: Vec<State> = Vec::new();

    for rating in TEST_RATINGS.iter() {
        state_history.push(card.state);
        let scheduled_cards = fsrs.schedule(card, now);

        card = scheduled_cards.select_card(*rating);
        log_history.push(card.clone().log.unwrap());
        now = card.due;
    }

    let expected: Vec<ReviewLog> = vec![
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 0,
            elapsed_days: 0,
            state: State::New,
            reviewed_date: string_to_utc("2022-11-29 12:30:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 5,
            elapsed_days: 0,
            state: State::Learning,
            reviewed_date: string_to_utc("2022-11-29 12:40:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 16,
            elapsed_days: 5,
            state: State::Review,
            reviewed_date: string_to_utc("2022-12-04 12:40:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 43,
            elapsed_days: 16,
            state: State::Review,
            reviewed_date: string_to_utc("2022-12-20 12:40:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 106,
            elapsed_days: 43,
            state: State::Review,
            reviewed_date: string_to_utc("2023-02-01 12:40:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 236,
            elapsed_days: 106,
            state: State::Review,
            reviewed_date: string_to_utc("2023-05-18 12:40:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Again,
            scheduled_days: 0,
            elapsed_days: 236,
            state: State::Review,
            reviewed_date: string_to_utc("2024-01-09 12:40:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Again,
            scheduled_days: 0,
            elapsed_days: 0,
            state: State::Relearning,
            reviewed_date: string_to_utc("2024-01-09 12:45:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 12,
            elapsed_days: 0,
            state: State::Relearning,
            reviewed_date: string_to_utc("2024-01-09 12:50:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 25,
            elapsed_days: 12,
            state: State::Review,
            reviewed_date: string_to_utc("2024-01-21 12:50:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 47,
            elapsed_days: 25,
            state: State::Review,
            reviewed_date: string_to_utc("2024-02-15 12:50:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 85,
            elapsed_days: 47,
            state: State::Review,
            reviewed_date: string_to_utc("2024-04-02 12:50:00 +0000 UTC"),
        },
        ReviewLog {
            rating: Rating::Good,
            scheduled_days: 147,
            elapsed_days: 85,
            state: State::Review,
            reviewed_date: string_to_utc("2024-06-26 12:50:00 +0000 UTC"),
        },
    ];
    assert_eq!(log_history, expected);
}
