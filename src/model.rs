#![allow(unused)]

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result};

const NUM_WEIGHTS: usize = 17;

#[derive(Debug, Clone)]
pub struct Weights(pub [f64; NUM_WEIGHTS]);

impl Default for Weights {
    fn default() -> Self {
        Weights([
            0.4, 0.6, 2.4, 5.8, 4.93, 0.94, 0.86, 0.01, 1.49, 0.14, 0.94, 2.18, 0.05, 0.34, 1.26, 0.29, 2.61,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Parameters {
    pub request_retention: f64,
    pub maximum_interval: f64,
    pub w: Weights,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            request_retention: 0.9,
            maximum_interval: 36500.0,
            w: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Card {
    pub due: chrono::DateTime<Utc>,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: u64,
    pub scheduled_days: u64,
    pub reps: u64,
    pub lapses: u64,
    pub state: State,
    pub last_review: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewLog {
    pub rating: Rating,
    pub scheduled_days: u64,
    pub elapsed_days: u64,
    pub review: DateTime<Utc>,
    pub state: State,
}

#[derive(Debug, Clone, Default)]
pub struct SchedulingCards {
    pub again: Card,
    pub hard: Card,
    pub good: Card,
    pub easy: Card,
}

impl SchedulingCards {
    pub fn init(&mut self, card: &Card) {
        self.again = card.clone();
        self.hard = card.clone();
        self.good = card.clone();
        self.easy = card.clone();
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SchedulingInfo {
    pub card: Card,
    pub review_log: ReviewLog,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, Serialize)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl From<Rating> for i8 {
    fn from(rating: Rating) -> i8 {
        match rating {
            Rating::Again => 1,
            Rating::Hard => 2,
            Rating::Good => 3,
            Rating::Easy => 4,
        }
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let rating_str = match self {
            Rating::Again => "Again",
            Rating::Hard => "Hard",
            Rating::Good => "Good",
            Rating::Easy => "Easy",
            _ => "unknown",
        };
        write!(f, "{rating_str}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize)]
pub enum State {
    #[default]
    New,
    Learning,
    Review,
    Relearning,
}

impl From<State> for i8 {
    fn from(rating: State) -> i8 {
        match rating {
            State::New => 0,
            State::Learning => 1,
            State::Review => 2,
            State::Relearning => 3,
        }
    }
}
