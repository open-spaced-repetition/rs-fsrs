use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug, Default, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum State {
    #[default]
    New = 0,
    Learning = 1,
    Review = 2,
    Relearning = 3,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub fn iter() -> std::slice::Iter<'static, Self> {
        static VARIANTS: [Rating; 4] = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];
        VARIANTS.iter()
    }
}

#[derive(Debug, Clone)]
pub struct ScheduledCards {
    pub cards: HashMap<Rating, Card>,
    pub now: DateTime<Utc>,
}

impl ScheduledCards {
    pub fn new(card: &Card, now: DateTime<Utc>) -> Self {
        let mut cards = HashMap::new();
        for rating in Rating::iter() {
            cards.insert(*rating, card.clone());
            let Some(card) = cards.get_mut(rating) else {
                continue;
            };
            card.update_state(*rating);
        }

        Self { cards, now }
    }

    pub fn select_card(&self, rating: Rating) -> Card {
        self.cards.get(&rating).unwrap().clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReviewLog {
    pub rating: Rating,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub state: State,
    pub reviewed_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    pub request_retention: f32,
    pub maximum_interval: i32,
    pub w: [f32; 17],
}

pub const DECAY: f32 = -0.5;
/// (9/10) ^ (1 / DECAY) - 1
pub const FACTOR: f32 = 19f32 / 81f32;

impl Default for Parameters {
    fn default() -> Self {
        Self {
            request_retention: 0.9,
            maximum_interval: 36500,
            w: [
                0.5701, 1.4436, 4.1386, 10.9355, 5.1443, 1.2006, 0.8627, 0.0362, 1.629, 0.1342,
                1.0166, 2.1174, 0.0839, 0.3204, 1.4676, 0.219, 2.8237,
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Card {
    pub due: DateTime<Utc>,
    pub stability: f32,
    pub difficulty: f32,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i32,
    pub lapses: i32,
    pub state: State,
    pub last_review: DateTime<Utc>,
    pub previous_state: State,
    pub log: Option<ReviewLog>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            due: Utc::now(),
            last_review: Utc::now(),
            ..Default::default()
        }
    }

    pub fn get_retrievability(&self) -> f32 {
        (1.0 + FACTOR * self.elapsed_days as f32 / self.stability).powf(DECAY)
    }

    pub fn save_log(&mut self, rating: Rating) {
        self.log = Some(ReviewLog {
            rating,
            elapsed_days: self.elapsed_days,
            scheduled_days: self.scheduled_days,
            state: self.previous_state,
            reviewed_date: self.last_review,
        });
    }

    pub fn update_state(&mut self, rating: Rating) {
        match (self.state, rating) {
            (State::New, Rating::Easy)
            | (State::Learning | State::Relearning, Rating::Good | Rating::Easy) => {
                self.state = State::Review
            }
            (State::New, _) => self.state = State::Learning,
            (State::Review, Rating::Again) => {
                self.lapses += 1;
                self.state = State::Relearning;
            }
            _ => {}
        }
    }
}
