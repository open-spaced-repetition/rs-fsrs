#[cfg(test)]
use crate::models::State;
#[cfg(test)]
use crate::{
    fsrs::FSRS,
    models::{Card, Parameters, Rating},
};
#[cfg(test)]
use chrono::Utc;

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

#[test]
fn test_interval() {
    let mut params = Parameters::default();
    params.w = WEIGHTS;

    let mut fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = Utc::now();
    let mut interval_history: Vec<i64> = Vec::new();

    for rating in TEST_RATINGS.iter() {
        fsrs.schedule(&mut card, now);
        card = fsrs.select_card(*rating);

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

    let mut fsrs = FSRS::new(params);
    let mut card = Card::new();
    let mut now = Utc::now();
    let mut state_history: Vec<State> = Vec::new();

    for rating in TEST_RATINGS.iter() {
        fsrs.schedule(&mut card, now);
        state_history.push(card.state);

        card = fsrs.select_card(*rating);
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
