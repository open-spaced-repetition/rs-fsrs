use crate::models::{Card, Rating, RecordLog, SchedulingInfo};
use crate::parameters::Parameters;
use crate::scheduler_basic::BasicScheduler;
use crate::ImplScheduler;

use chrono::{DateTime, Utc};

#[derive(Debug, Default, Clone, Copy)]
pub struct FSRS {
    parameters: Parameters,
}

impl FSRS {
    pub const fn new(parameters: Parameters) -> Self {
        Self { parameters }
    }

    pub fn scheduler(&self, card: Card, now: DateTime<Utc>) -> BasicScheduler {
        BasicScheduler::new(self.parameters, card, now)
    }

    pub fn repeat(&self, card: Card, now: DateTime<Utc>) -> RecordLog {
        self.scheduler(card, now).preview()
    }

    pub fn next(&self, card: Card, now: DateTime<Utc>, rating: Rating) -> SchedulingInfo {
        self.scheduler(card, now).review(rating)
    }
}
