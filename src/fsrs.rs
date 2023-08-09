#![allow(unused)]
use chrono::{DateTime, Duration, Utc};

use crate::{fsrs::Rating::*, fsrs::State::*, model::*};
use std::collections::HashMap;

// L8
impl Parameters {
    pub fn repeat(&self, mut card: Card, now: DateTime<Utc>) -> HashMap<Rating, SchedulingInfo> {
        if card.state == State::New {
            card.elapsed_days = 0;
        } else {
            let elapsed_time = now.signed_duration_since(card.last_review);
            let elapsed_days = elapsed_time.num_seconds() / 86400;
            card.elapsed_days = elapsed_days as u64;
        }
        card.last_review = now;
        card.reps += 1;
        let mut scheduling_cards = SchedulingCards::init(&card) //
            .update_state(&card.state); // L18

        match card.state {
            State::New => {
                self.init_ds(&mut scheduling_cards);

                scheduling_cards.again.due = now.checked_add_signed(Duration::seconds(60)).unwrap();
                scheduling_cards.hard.due = now.checked_add_signed(Duration::seconds(300)).unwrap();
                scheduling_cards.good.due = now.checked_add_signed(Duration::seconds(600)).unwrap();

                let easy_interval = self.next_interval(scheduling_cards.easy.stability);
                scheduling_cards.easy.scheduled_days = easy_interval.round() as u64;
                scheduling_cards.easy.due =
                    now + Duration::seconds((easy_interval * 86400.0) as i64);
            }
            State::Learning | State::Relearning => {
                let hard_interval = 0.0;
                let good_interval = self.next_interval(scheduling_cards.good.stability);
                let easy_interval = self
                    .next_interval(scheduling_cards.easy.stability)
                    .max(good_interval + 1.0);

                scheduling_cards.schedule(now, hard_interval, good_interval, easy_interval);
            }
            State::Review => {
                let interval = card.elapsed_days as f64;
                let last_d = card.difficulty;
                let last_s = card.stability;
                let retrievability = (1.0 + interval / (9.0 * last_s)).powf(-1.0);
                self.next_ds(&mut scheduling_cards, last_d, last_s, retrievability);

                let hard_interval = self.next_interval(scheduling_cards.hard.stability);
                let good_interval = self.next_interval(scheduling_cards.good.stability);
                let hard_interval = hard_interval.min(good_interval);
                let good_interval = good_interval.max(hard_interval + 1.0);
                let easy_interval = self
                    .next_interval(scheduling_cards.easy.stability)
                    .max(good_interval + 1.0);

                scheduling_cards.schedule(now, hard_interval, good_interval, easy_interval);
            }
        }

        scheduling_cards.record_log(&card, &now)
    }
} // L51
  // L53
impl SchedulingCards {
    fn update_state(&mut self, state: &State) -> SchedulingCards {
        use State::*;
        match state {
            New => {
                self.again.state = Learning;
                self.hard.state = Learning;
                self.good.state = Learning;
                self.easy.state = Review;
                self.again.lapses += 1;
            }
            Learning | Relearning => {
                self.again.state = state.clone();
                self.hard.state = state.clone();
                self.good.state = Review;
                self.easy.state = Review;
            }
            Review => {
                self.again.state = Relearning;
                self.hard.state = Review;
                self.good.state = Review;
                self.easy.state = Review;
                self.again.lapses += 1;
            }
        }
        self.clone()
    }
}

// L75
impl SchedulingCards {
    fn schedule(
        &mut self,
        now: DateTime<Utc>,
        hard_interval: f64,
        good_interval: f64,
        easy_interval: f64,
    ) {
        self.again.scheduled_days = 0;
        self.hard.scheduled_days = hard_interval as u64;
        self.good.scheduled_days = good_interval as u64;
        self.easy.scheduled_days = easy_interval as u64;
        self.again.due = now + Duration::minutes(5);
        self.hard.due = now
            + if hard_interval > 0.0 {
                Duration::days(hard_interval as i64)
            } else {
                Duration::minutes(10)
            };
        self.good.due = now + Duration::days(good_interval as i64);
        self.easy.due = now + Duration::days(easy_interval as i64);
    }
}
impl SchedulingCards {
    // L86
    fn record_log(&self, card: &Card, now: &DateTime<Utc>) -> HashMap<Rating, SchedulingInfo> {
        use Rating::*;
        HashMap::from([
            (
                Again,
                SchedulingInfo {
                    card: self.again.clone(),
                    review_log: ReviewLog {
                        rating: Again,
                        scheduled_days: self.again.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
            (
                Hard,
                SchedulingInfo {
                    card: self.hard.clone(),
                    review_log: ReviewLog {
                        rating: Hard,
                        scheduled_days: self.hard.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
            (
                Good,
                SchedulingInfo {
                    card: self.good.clone(),
                    review_log: ReviewLog {
                        rating: Good,
                        scheduled_days: self.good.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
            (
                Easy,
                SchedulingInfo {
                    card: self.easy.clone(),
                    review_log: ReviewLog {
                        rating: Easy,
                        scheduled_days: self.easy.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
        ])
    }
}

impl Parameters {
    // L120
    fn init_ds(&self, s: &mut SchedulingCards) {
        s.again.difficulty = self.init_difficulty(Again);
        s.again.stability = self.init_stability(Again);
        s.hard.difficulty = self.init_difficulty(Hard);
        s.hard.stability = self.init_stability(Hard);
        s.good.difficulty = self.init_difficulty(Good);
        s.good.stability = self.init_stability(Good);
        s.easy.difficulty = self.init_difficulty(Easy);
        s.easy.stability = self.init_stability(Easy);
    }
    // 131
    fn next_ds(&self, s: &mut SchedulingCards, last_d: f64, last_s: f64, retrievability: f64) {
        s.again.difficulty = self.next_difficulty(last_d, Again);
        s.again.stability = self.next_forget_stability(s.again.difficulty, last_s, retrievability);
        s.hard.difficulty = self.next_difficulty(last_d, Hard);
        s.hard.stability =
            self.next_recall_stability(s.hard.difficulty, last_s, retrievability, Hard);
        s.good.difficulty = self.next_difficulty(last_d, Good);
        s.good.stability =
            self.next_recall_stability(s.good.difficulty, last_s, retrievability, Good);
        s.easy.difficulty = self.next_difficulty(last_d, Easy);
        s.easy.stability =
            self.next_recall_stability(s.easy.difficulty, last_s, retrievability, Easy);
    }
    // 142
    fn init_stability(&self, r: Rating) -> f64 {
        self.w.0[(r as usize) - 1].max(0.1)
    }

    fn init_difficulty(&self, r: Rating) -> f64 {
        constrain_difficulty(self.w.0[4] - self.w.0[5] * f64::from(r as i8 - 3))
    }

    // L149-177
    fn next_interval(&self, s: f64) -> f64 {
        let new_interval = s * 9.0 * (1.0 / self.request_retention - 1.0);
        new_interval.round().max(1.0).min(self.maximum_interval)
    }

    fn next_difficulty(&self, d: f64, r: Rating) -> f64 {
        let next_d = d - self.w.0[6] * f64::from(r as i8 - 3);
        constrain_difficulty(self.mean_reversion(self.w.0[4], next_d))
    }

    fn mean_reversion(&self, init: f64, current: f64) -> f64 {
        self.w.0[7] * init + (1.0 - self.w.0[7]) * current
    }

    fn next_recall_stability(&self, d: f64, s: f64, r: f64, rating: Rating) -> f64 {
        let hard_penalty = if rating == Hard { self.w.0[15] } else { 1.0 };
        let easy_bonus = if rating == Easy { self.w.0[16] } else { 1.0 };
        s * (1.0
            + f64::exp(self.w.0[8])
                * (11.0 - d)
                * f64::powf(s, -self.w.0[9])
                * (f64::exp((1.0 - r) * self.w.0[10]) - 1.0)
                * hard_penalty
                * easy_bonus)
    }

    fn next_forget_stability(&self, d: f64, s: f64, r: f64) -> f64 {
        self.w.0[11]
            * f64::powf(d, -self.w.0[12])
            * (f64::powf(s + 1.0, self.w.0[13]) - 1.0)
            * f64::exp((1.0 - r) * self.w.0[14])
    }
}
fn constrain_difficulty(d: f64) -> f64 {
    d.max(1.0).min(10.0)
}
#[cfg(test)]
mod test {
    use super::*;

    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn test_repeat() {
        let p = Parameters {
            w: Weights([
                1.14, 1.01, 5.44, 14.67, 5.3024, 1.5662, 1.2503, 0.0028, 1.5489, 0.1763, 0.9953,
                2.7473, 0.0179, 0.3105, 0.3976, 0.0, 2.0902,
            ]),
            ..Default::default()
        };
        let mut card = Card::default();
        let mut now = Utc
            .with_ymd_and_hms(2022, 11, 29, 12, 30, 0)
            .single()
            .unwrap();
        // empty int vec
        let mut ivl_vec = vec![];
        let mut state_vec = vec![];
        let mut scheduling_cards = p.repeat(card, now);
        let mut schedule = serde_json::to_string_pretty(&scheduling_cards).unwrap();
        println!("{}", schedule);

        let ratings = [
            Good, Good, Good, Good, Good, Good, Again, Again, Good, Good, Good, Good, Good,
        ];
        for rating in ratings {
            card = scheduling_cards[&rating].card.clone();
            let revlog = scheduling_cards[&rating].review_log.clone();
            ivl_vec.push(card.scheduled_days);
            state_vec.push(revlog.state);
            now = card.due;
            scheduling_cards = p.repeat(card, now);
            schedule = serde_json::to_string_pretty(&scheduling_cards).unwrap();
            println!("{}", schedule);
        }

        dbg!(&ivl_vec);
        dbg!(&state_vec);

        assert_eq!(
            ivl_vec,
            vec![0, 5, 16, 43, 106, 236, 0, 0, 12, 25, 47, 85, 147]
        );
        assert_eq!(
            state_vec,
            vec![
                New, Learning, Review, Review, Review, Review, Review, Relearning, Relearning,
                Review, Review, Review, Review
            ]
        );
    }
}
