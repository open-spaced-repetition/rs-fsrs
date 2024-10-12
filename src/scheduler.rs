use chrono::{DateTime, Utc};

use crate::models::State::*;
use crate::Seed;
use crate::{
    models::{RecordLog, SchedulingInfo},
    Card, Parameters, Rating, ReviewLog,
};

#[derive(Debug, Clone)]
pub struct Scheduler {
    pub parameters: Parameters,
    pub last: Card,
    pub current: Card,
    pub now: DateTime<Utc>,
    pub next: RecordLog,
}

impl Scheduler {
    pub fn new(mut parameters: Parameters, card: Card, now: DateTime<Utc>) -> Self {
        let mut current_card: Card = card.clone();
        current_card.elapsed_days = match card.state {
            New => 0,
            _ => (now - card.last_review).num_days(),
        };
        current_card.last_review = now;
        current_card.reps += 1;
        Self::init_seed(&mut parameters, &current_card);

        Self {
            parameters,
            last: card,
            current: current_card,
            now,
            next: RecordLog::new(),
        }
    }

    pub const fn build_log(&self, rating: Rating) -> ReviewLog {
        ReviewLog {
            rating,
            state: self.current.state,
            elapsed_days: self.current.elapsed_days,
            scheduled_days: self.current.scheduled_days,
            reviewed_date: self.now,
        }
    }

    fn init_seed(parameters: &mut Parameters, current: &Card) {
        let time = Utc::now().timestamp_millis();
        let reps = current.reps;
        let mul = current.difficulty * current.stability;
        parameters.seed = Seed::new(format!("{}_{}_{}", time, reps, mul));
    }
}

pub trait ImplScheduler {
    fn preview(&mut self) -> RecordLog {
        let mut log = RecordLog::new();
        for rating in Rating::iter() {
            log.insert(*rating, self.review(*rating));
        }
        log
    }
    fn review(&mut self, rating: Rating) -> SchedulingInfo;
}
