use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum State {
    #[default]
    New = 0,
    Learning = 1,
    Review = 2,
    Relearning = 3,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
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
            if let Some(card) = cards.get_mut(rating) {
                card.update_state(*rating);
            }
        }

        Self { cards, now }
    }

    pub fn select_card(&self, rating: Rating) -> Card {
        self.cards.get(&rating).unwrap().clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            request_retention: 0.9,
            maximum_interval: 36500,
            w: [
                0.4, 0.6, 2.4, 5.8, 4.93, 0.94, 0.86, 0.01, 1.49, 0.14, 0.94, 2.18, 0.05, 0.34,
                1.26, 0.29, 2.61,
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
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
        (1.0 + self.elapsed_days as f32 / (9.0 * self.stability)).powf(-1.0)
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
        match self.state {
            State::New => match rating {
                Rating::Again => self.lapses += 1,
                Rating::Easy => self.state = State::Review,
                _ => self.state = State::Learning,
            },
            State::Learning | State::Relearning => {
                if rating == Rating::Good || rating == Rating::Easy {
                    self.state = State::Review
                }
            }
            State::Review => {
                if rating == Rating::Again {
                    self.lapses += 1;
                    self.state = State::Relearning;
                }
            }
        }
    }
}
