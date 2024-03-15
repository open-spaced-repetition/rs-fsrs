use crate::models::Rating;
use crate::models::Rating::{Again, Easy, Good, Hard};
use crate::models::State::{Learning, New, Relearning, Review};
use crate::models::*;
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
                self.init_difficulty_stability(&mut output_cards);

                self.set_due(&mut output_cards, Again, Duration::minutes(1));
                self.set_due(&mut output_cards, Hard, Duration::minutes(5));
                self.set_due(&mut output_cards, Good, Duration::minutes(10));

                let easy_interval = self.next_interval(&mut output_cards, Easy).unwrap();
                self.set_scheduled_days(&mut output_cards, Easy, easy_interval);
                self.set_due(&mut output_cards, Easy, Duration::days(easy_interval));
            }
            Learning | Relearning => {
                self.set_scheduled_days(&mut output_cards, Again, 0);
                self.set_due(&mut output_cards, Again, Duration::minutes(5));

                self.set_scheduled_days(&mut output_cards, Hard, 0);
                self.set_due(&mut output_cards, Hard, Duration::minutes(10));

                let good_interval = self.next_interval(&mut output_cards, Good).unwrap();

                let easy_interval =
                    (good_interval + 1).max(self.next_interval(&mut output_cards, Easy).unwrap());

                self.set_scheduled_days(&mut output_cards, Good, good_interval);
                self.set_due(&mut output_cards, Good, Duration::days(good_interval));

                self.set_scheduled_days(&mut output_cards, Easy, easy_interval);
                self.set_due(&mut output_cards, Easy, Duration::days(easy_interval));
            }
            Review => {
                self.next_stability(&mut output_cards);
                self.next_difficulty(&mut output_cards);

                let mut hard_interval = self.next_interval(&mut output_cards, Hard).unwrap();
                let mut good_interval = self.next_interval(&mut output_cards, Good).unwrap();
                let mut easy_interval = self.next_interval(&mut output_cards, Easy).unwrap();

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

    fn init_difficulty_stability(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            let rating_int: i32 = *rating as i32;
            let Some(card) = output_cards.cards.get_mut(rating) else {
                continue;
            };
            card.difficulty = self.params.w[5]
                .mul_add(-(rating_int as f32 - 3.0), self.params.w[4])
                .clamp(1.0, 10.0);
            card.stability = self.params.w[(rating_int - 1) as usize].max(0.1);
        }
    }

    #[allow(clippy::suboptimal_flops)]
    fn next_interval(
        &self,
        output_cards: &mut ScheduledCards,
        rating: Rating,
    ) -> Result<i64, String> {
        let Some(card) = output_cards.cards.get_mut(&rating) else {
            return Err("Failed to retrieve card from output_cards".to_string());
        };
        let new_interval =
            card.stability / FACTOR * (self.params.request_retention.powf(1.0 / DECAY) - 1.0);
        Ok((new_interval.round() as i64).clamp(1, self.params.maximum_interval as i64))
    }

    fn next_stability(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            if rating == &Again {
                self.next_forget_stability(output_cards);
            } else {
                self.next_recall_stability(output_cards, *rating);
            }
        }
    }

    fn next_recall_stability(&self, output_cards: &mut ScheduledCards, rating: Rating) {
        let modifier = match rating {
            Hard => self.params.w[15],
            Easy => self.params.w[16],
            _ => 1.0,
        };

        let Some(card) = output_cards.cards.get_mut(&rating) else {
            return;
        };
        let retrievability = card.get_retrievability();
        card.stability = card.stability
            * (((self.params.w[8]).exp()
                * (11.0 - card.difficulty)
                * card.stability.powf(-self.params.w[9])
                * (((1.0 - retrievability) * self.params.w[10]).exp_m1()))
            .mul_add(modifier, 1.0));
    }

    fn next_forget_stability(&self, output_cards: &mut ScheduledCards) {
        let Some(card) = output_cards.cards.get_mut(&Again) else {
            return;
        };
        let retrievability = card.get_retrievability();
        card.stability = self.params.w[11]
            * card.difficulty.powf(-self.params.w[12])
            * ((card.stability + 1.0).powf(self.params.w[13]) - 1.0)
            * f32::exp((1.0 - retrievability) * self.params.w[14])
    }

    fn next_difficulty(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            let rating_int = *rating as i32;
            let Some(mut card) = output_cards.cards.remove(rating) else {
                continue;
            };
            let next_difficulty =
                self.params.w[6].mul_add(-(rating_int as f32 - 3.0), card.difficulty);
            let mean_reversion = self.mean_reversion(self.params.w[4], next_difficulty);
            card.difficulty = mean_reversion.clamp(1.0, 10.0);
            output_cards.cards.insert(*rating, card);
        }
    }

    fn mean_reversion(&self, initial: f32, current: f32) -> f32 {
        self.params.w[7].mul_add(initial, (1.0 - self.params.w[7]) * current)
    }
}
