use crate::models::Rating;
use crate::models::Rating::{Again, Easy, Good, Hard};
use crate::models::State::{Learning, New, Relearning, Review};
use crate::models::*;
use crate::parameters::Parameters;

use chrono::{DateTime, Duration, Utc};
use std::cmp;

#[derive(Debug, Default, Clone, Copy)]
pub struct FSRS {
    params: Parameters,
}

impl FSRS {
    pub const fn new(params: Parameters) -> Self {
        Self { params }
    }

    pub fn schedule(&self, mut card: Card, now: DateTime<Utc>) -> ScheduledCards {
        card.reps += 1;
        card.previous_state = card.state;

        card.elapsed_days = match card.state {
            New => 0,
            _ => (now - card.last_review).num_days(),
        };
        card.last_review = now;

        let mut output_cards = ScheduledCards::new(&card, now);

        match card.state {
            New => {
                self.params.init_difficulty_stability(&mut output_cards);

                self.set_due(&mut output_cards, Again, Duration::minutes(1));
                self.set_due(&mut output_cards, Hard, Duration::minutes(5));
                self.set_due(&mut output_cards, Good, Duration::minutes(10));

                let easy_interval = self.params.next_interval(&mut output_cards, Easy).unwrap();
                self.set_scheduled_days(&mut output_cards, Easy, easy_interval);
                self.set_due(&mut output_cards, Easy, Duration::days(easy_interval));
            }
            Learning | Relearning => {
                self.params.next_stability(&mut output_cards, card.state);
                self.params.next_difficulty(&mut output_cards);

                self.set_scheduled_days(&mut output_cards, Again, 0);
                self.set_due(&mut output_cards, Again, Duration::minutes(5));

                self.set_scheduled_days(&mut output_cards, Hard, 0);
                self.set_due(&mut output_cards, Hard, Duration::minutes(10));

                let good_interval = self.params.next_interval(&mut output_cards, Good).unwrap();

                let easy_interval = (good_interval + 1)
                    .max(self.params.next_interval(&mut output_cards, Easy).unwrap());

                self.set_scheduled_days(&mut output_cards, Good, good_interval);
                self.set_due(&mut output_cards, Good, Duration::days(good_interval));

                self.set_scheduled_days(&mut output_cards, Easy, easy_interval);
                self.set_due(&mut output_cards, Easy, Duration::days(easy_interval));
            }
            Review => {
                self.params.next_stability(&mut output_cards, card.state);
                self.params.next_difficulty(&mut output_cards);

                let mut hard_interval = self.params.next_interval(&mut output_cards, Hard).unwrap();
                let mut good_interval = self.params.next_interval(&mut output_cards, Good).unwrap();
                let mut easy_interval = self.params.next_interval(&mut output_cards, Easy).unwrap();

                hard_interval = cmp::min(hard_interval, good_interval);
                good_interval = cmp::max(good_interval, hard_interval + 1);
                easy_interval = cmp::max(good_interval + 1, easy_interval);

                self.set_scheduled_days(&mut output_cards, Again, 0);
                self.set_due(&mut output_cards, Again, Duration::minutes(5));

                self.set_scheduled_days(&mut output_cards, Hard, hard_interval);
                self.set_due(&mut output_cards, Hard, Duration::days(hard_interval));

                self.set_scheduled_days(&mut output_cards, Good, good_interval);
                self.set_due(&mut output_cards, Good, Duration::days(good_interval));

                self.set_scheduled_days(&mut output_cards, Easy, easy_interval);
                self.set_due(&mut output_cards, Easy, Duration::days(easy_interval));
            }
        }
        self.save_logs(&mut output_cards);
        output_cards
    }

    fn set_due(&self, output_cards: &mut ScheduledCards, rating: Rating, duration: Duration) {
        let Some(card) = output_cards.cards.get_mut(&rating) else {
            return;
        };
        card.due = output_cards.now + duration;
    }

    fn set_scheduled_days(&self, output_cards: &mut ScheduledCards, rating: Rating, interval: i64) {
        let Some(card) = output_cards.cards.get_mut(&rating) else {
            return;
        };
        card.scheduled_days = interval;
    }

    fn save_logs(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            let Some(card) = output_cards.cards.get_mut(rating) else {
                continue;
            };
            card.save_log(*rating);
        }
    }
}
