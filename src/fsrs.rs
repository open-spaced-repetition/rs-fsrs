use crate::fsrs::Rating;
use crate::fsrs::Rating::{Again, Easy, Good, Hard};
use crate::fsrs::State::{Learning, New, Relearning, Review};
use crate::models::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
pub struct FSRS<'a> {
    params: Parameters,
    output_cards: HashMap<&'a Rating, Card>,
    now: DateTime<Utc>,
}

impl Default for FSRS<'_> {
    fn default() -> Self {
        Self {
            params: Parameters::default(),
            output_cards: HashMap::new(),
            now: Utc::now(),
        }
    }
}

impl FSRS<'_> {
    pub fn new(params: Parameters) -> Self {
        Self {
            params,
            output_cards: HashMap::new(),
            now: Utc::now(),
        }
    }

    pub fn schedule(&mut self, card: &mut Card, now: DateTime<Utc>) {
        card.reps += 1;
        self.now = now;
        self.output_cards = HashMap::new();
        if card.state == New {
            card.elapsed_days = 0;
        } else {
            card.elapsed_days = (now - card.last_review).num_days();
        }
        card.last_review = now;

        for rating in Rating::iter() {
            self.output_cards.insert(&rating, card.clone());
            if let Some(card) = self.output_cards.get_mut(rating) {
                card.update_state(*rating);
            }
        }
        match card.state {
            New => {
                self.init_difficulty_stability();
                self.set_due(Again, Duration::minutes(1));
                self.set_due(Hard, Duration::minutes(5));
                self.set_due(Good, Duration::minutes(10));

                let easy_interval = self.next_interval(Easy).unwrap();
                self.set_scheduled_days(Easy, easy_interval);
                self.set_due(Easy, Duration::days(easy_interval));
            }
            Learning | Relearning => {
                self.set_scheduled_days(Again, 0);
                self.set_due(Again, Duration::minutes(5));

                self.set_scheduled_days(Hard, 0);
                self.set_due(Hard, Duration::minutes(10));

                let good_interval = self.next_interval(Good).unwrap();

                let easy_interval =
                    (good_interval + 1).max(self.next_interval(Easy).unwrap());

                self.set_scheduled_days(Good, good_interval);
                self.set_due(Good, Duration::days(good_interval));

                self.set_scheduled_days(Easy, easy_interval);
                self.set_due(Hard, Duration::days(easy_interval));
            }
            Review => {
                self.next_difficulty();
                self.next_stability();

                let mut hard_interval = self.next_interval(Hard).unwrap();
                let mut good_interval = self.next_interval(Good).unwrap();
                let mut easy_interval = self.next_interval(Easy).unwrap();

                hard_interval = hard_interval.min(good_interval);
                good_interval = good_interval.max(hard_interval + 1);
                easy_interval = easy_interval.max(good_interval + 1);

                self.set_scheduled_days(Again, 0);
                self.set_due(Again, Duration::minutes(5));

                self.set_scheduled_days(Good, good_interval);
                self.set_due(Good, Duration::days(good_interval));

                self.set_scheduled_days(Easy, easy_interval);
                self.set_due(Hard, Duration::days(easy_interval));
            }
        }
    }

    pub fn select_card(&mut self, rating: Rating) -> Card {
        return self.output_cards.remove(&rating).unwrap();
    }

    fn set_due(&mut self, rating: Rating, duration: Duration) {
        if let Some(card) = self.output_cards.get_mut(&rating) {
            card.due = self.now + duration;
        }
    }

    fn set_scheduled_days(&mut self, rating: Rating, interval: i64) {
        if let Some(card) = self.output_cards.get_mut(&rating) {
            card.scheduled_days = interval;
        }
    }

    fn init_difficulty_stability(&mut self) {
        for rating in Rating::iter() {
            let rating_int: i32 = *rating as i32;
            if let Some(card) = self.output_cards.get_mut(rating) {
                card.difficulty = (self.params.w[4] - self.params.w[5] * (rating_int as f32 - 3.0))
                    .max(1.0)
                    .min(10.0);
                card.stability = self.params.w[(rating_int - 1) as usize].max(0.1);
            }
        }
    }

    fn next_interval(&mut self, rating: Rating) -> Result<i64, String> {
        if let Some(card) = self.output_cards.get_mut(&rating) {
            let new_interval = card.stability * 9.0 * (1.0 / self.params.request_retention - 1.0);
            return Ok((new_interval.round() as i64)
                .max(1)
                .min(self.params.maximum_interval as i64));
        }
        Err("Failed to retrieve card from output_cards".to_string())
    }

    fn next_stability(&mut self) {
        for rating in Rating::iter() {
            if rating == &Again {
                self.next_forget_stability();
            } else {
                self.next_recall_stability(*rating);
            }
        }
    }

    fn next_recall_stability(&mut self, rating: Rating) {
        let mut modifier: f32 = 1.0;
        if rating == Hard {
            modifier = self.params.w[15];
        }
        if rating == Easy {
            modifier = self.params.w[16];
        }

        if let Some(card) = self.output_cards.get_mut(&rating) {
            let retrievability = card.get_retrievability();

            card.stability = card.stability
                * (1.0
                    + f32::exp(self.params.w[8])
                        * (11.0 - card.difficulty)
                        * card.stability.powf(-self.params.w[9])
                        * (f32::exp((1.0 - retrievability) * self.params.w[10]) - 1.0)
                        * modifier);
        }
    }

    fn next_forget_stability(&mut self) {
        if let Some(card) = self.output_cards.get_mut(&Again) {
            let retrievability = card.get_retrievability();
            card.stability = self.params.w[11]
                * card.difficulty.powf(-self.params.w[12])
                * ((card.stability + 1.0).powf(self.params.w[13]) - 1.0)
                * f32::exp((1.0 - retrievability) * self.params.w[14])
        }
    }

    fn next_difficulty(&mut self) {
        for rating in Rating::iter() {
            let rating_int = *rating as i32;
            if let Some(mut card) = self.output_cards.remove(rating) {
                let next_difficulty =
                    card.difficulty - (self.params.w[6] * (rating_int as f32 - 3.0));
                let mean_reversion = self.mean_reversion(self.params.w[4], next_difficulty);
                card.difficulty = mean_reversion.max(1.0).min(10.0);
                self.output_cards.insert(rating, card);
            }
        }
    }

    fn mean_reversion(&self, initial: f32, current: f32) -> f32 {
        return self.params.w[7] * initial + (1.0 - self.params.w[7]) * current;
    }
}
