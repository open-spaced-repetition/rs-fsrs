#![allow(unused)]
use chrono::{DateTime, Duration, Utc};

use crate::{fsrs::Rating::*, model::*};
use std::collections::HashMap;

// L8
impl Parameters {
    pub fn repeat(
        &self,
        mut card: Card,
        now: chrono::DateTime<Utc>,
    ) -> HashMap<Rating, SchedulingInfo> {
        if card.state == State::New {
            card.elapsed_days = 0;
        } else {
            let elapsed_time = now.signed_duration_since(card.last_review);
            let elapsed_days = elapsed_time.num_seconds() / 86400;
            card.elapsed_days = elapsed_days as u64;
        }
        card.last_review = now;
        card.reps += 1;
        let mut scheduling_cards = SchedulingCards::default();
        scheduling_cards.init(&card);
        scheduling_cards.update_state(&card.state); // L18

        match card.state {
            State::New => {
                self.init_ds(&mut scheduling_cards);

                scheduling_cards.again.due = now.checked_add_signed(Duration::seconds(60)).unwrap();
                scheduling_cards.hard.due = now.checked_add_signed(Duration::seconds(300)).unwrap();
                scheduling_cards.good.due = now.checked_add_signed(Duration::seconds(600)).unwrap();

                let easy_interval =
                    self.next_interval(scheduling_cards.easy.stability * self.easy_bonus);
                scheduling_cards.easy.scheduled_days = easy_interval.round() as u64;
                scheduling_cards.easy.due =
                    now + Duration::seconds((easy_interval * 86400.0) as i64);
            }
            State::Learning | State::Relearning => {
                let hard_interval = self.next_interval(scheduling_cards.hard.stability);
                let good_interval = f64::max(
                    self.next_interval(scheduling_cards.good.stability),
                    hard_interval + 1.0,
                );
                let easy_interval = f64::max(
                    self.next_interval(scheduling_cards.easy.stability * self.easy_bonus),
                    good_interval + 1.0,
                );

                scheduling_cards.schedule(now, hard_interval, good_interval, easy_interval);
            }
            State::Review => {
                let interval = card.elapsed_days as f64;
                let last_d = card.difficulty;
                let last_s = card.stability;
                let retrievability = f64::exp(f64::ln(0.9) * interval / last_s);
                self.next_ds(&mut scheduling_cards, last_d, last_s, retrievability);

                let hard_interval = self.next_interval(last_s * self.hard_factor);
                let good_interval = self.next_interval(scheduling_cards.good.stability);
                let hard_interval = f64::min(hard_interval, good_interval);
                let good_interval = f64::max(good_interval, hard_interval + 1.0);
                let easy_interval = f64::max(
                    self.next_interval(scheduling_cards.easy.stability * self.easy_bonus),
                    good_interval + 1.0,
                );

                scheduling_cards.schedule(now, hard_interval, good_interval, easy_interval);
            }
        }

        scheduling_cards.record_log(&card, &now)
    }
} // L51
  // L53
impl SchedulingCards {
    fn update_state(&mut self, state: &State) {
        match state {
            State::New => {
                self.again.state = State::Learning;
                self.hard.state = State::Learning;
                self.good.state = State::Learning;
                self.easy.state = State::Review;
                self.again.lapses += 1;
            }
            State::Learning | State::Relearning => {
                self.again.state = state.clone();
                self.hard.state = State::Review;
                self.good.state = State::Review;
                self.easy.state = State::Review;
            }
            State::Review => {
                self.again.state = State::Relearning;
                self.hard.state = State::Review;
                self.good.state = State::Review;
                self.easy.state = State::Review;
                self.again.lapses += 1;
            }
        }
    }
}

// L75
impl SchedulingCards {
    fn schedule(
        &mut self,
        now: chrono::DateTime<chrono::Utc>,
        hard_interval: f64,
        good_interval: f64,
        easy_interval: f64,
    ) {
        self.again.scheduled_days = 0;
        self.hard.scheduled_days = hard_interval as u64;
        self.good.scheduled_days = good_interval as u64;
        self.easy.scheduled_days = easy_interval as u64;
        self.again.due = now + chrono::Duration::minutes(5);
        self.hard.due = now + chrono::Duration::days(hard_interval as i64);
        self.good.due = now + chrono::Duration::days(good_interval as i64);
        self.easy.due = now + chrono::Duration::days(easy_interval as i64);
    }
}
impl SchedulingCards {
    // L86
    fn record_log(
        &self,
        card: &Card,
        now: &chrono::DateTime<chrono::Utc>,
    ) -> HashMap<Rating, SchedulingInfo> {
        HashMap::from([
            (
                Rating::Again,
                SchedulingInfo {
                    card: self.again.clone(),
                    review_log: ReviewLog {
                        rating: Rating::Again,
                        scheduled_days: self.again.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
            (
                Rating::Hard,
                SchedulingInfo {
                    card: self.hard.clone(),
                    review_log: ReviewLog {
                        rating: Rating::Hard,
                        scheduled_days: self.hard.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
            (
                Rating::Good,
                SchedulingInfo {
                    card: self.good.clone(),
                    review_log: ReviewLog {
                        rating: Rating::Good,
                        scheduled_days: self.good.scheduled_days,
                        elapsed_days: card.elapsed_days,
                        review: *now,
                        state: card.state.clone(),
                    },
                },
            ),
            (
                Rating::Easy,
                SchedulingInfo {
                    card: self.easy.clone(),
                    review_log: ReviewLog {
                        rating: Rating::Easy,
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
        s.hard.stability = self.next_recall_stability(s.hard.difficulty, last_s, retrievability);
        s.good.difficulty = self.next_difficulty(last_d, Good);
        s.good.stability = self.next_recall_stability(s.good.difficulty, last_s, retrievability);
        s.easy.difficulty = self.next_difficulty(last_d, Easy);
        s.easy.stability = self.next_recall_stability(s.easy.difficulty, last_s, retrievability);
    }
    // 142
    fn init_stability(&self, r: Rating) -> f64 {
        f64::max(self.w.0[0] + self.w.0[1] * f64::from(r as i8), 0.1)
    }

    fn init_difficulty(&self, r: Rating) -> f64 {
        constrain_difficulty(self.w.0[2] + self.w.0[3] * (f64::from(r as i8) - 2_f64))
    }

    // L149-177
    fn next_interval(&self, s: f64) -> f64 {
        let new_interval = s * f64::ln(self.request_retention) / f64::ln(0.9);
        new_interval.round().max(1.0).min(self.maximum_interval)
    }

    fn next_difficulty(&self, d: f64, r: Rating) -> f64 {
        let next_d = d + self.w.0[4] * f64::from(r as i8 - 2);
        constrain_difficulty(self.mean_reversion(self.w.0[2], next_d))
    }

    fn mean_reversion(&self, init: f64, current: f64) -> f64 {
        self.w.0[5] * init + (1.0 - self.w.0[5]) * current
    }

    fn next_recall_stability(&self, d: f64, s: f64, r: f64) -> f64 {
        s * (1.0
            + f64::exp(self.w.0[6])
                * (11.0 - d)
                * f64::powf(s, self.w.0[7])
                * (f64::exp((1.0 - r) * self.w.0[8]) - 1.0))
    }

    fn next_forget_stability(&self, d: f64, s: f64, r: f64) -> f64 {
        self.w.0[9]
            * f64::powf(d, self.w.0[10])
            * f64::powf(s, self.w.0[11])
            * f64::exp((1.0 - r) * self.w.0[12])
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
        let p = Parameters::default();
        let card = Card::default();
        let now = Utc
            .with_ymd_and_hms(2022, 11, 29, 12, 30, 0)
            .single()
            .unwrap();
        let scheduling_cards = p.repeat(card, now);
        let schedule = serde_json::to_string(&scheduling_cards).unwrap();
        println!("{}", schedule);

        let card = scheduling_cards.get(&Rating::Good).unwrap().card.clone();
        let now = card.due;
        let scheduling_cards = p.repeat(card, now);
        let schedule = serde_json::to_string(&scheduling_cards).unwrap();
        println!("{}", schedule);

        let card = scheduling_cards.get(&Rating::Good).unwrap().card.clone();
        let now = card.due;
        let scheduling_cards = p.repeat(card, now);
        let schedule = serde_json::to_string(&scheduling_cards).unwrap();
        println!("{}", schedule);

        let card = scheduling_cards.get(&Rating::Again).unwrap().card.clone();
        let now = card.due;
        let scheduling_cards = p.repeat(card, now);
        let schedule = serde_json::to_string(&scheduling_cards).unwrap();
        println!("{}", schedule);

        let card = scheduling_cards.get(&Rating::Good).unwrap().card.clone();
        let now = card.due;
        let scheduling_cards = p.repeat(card, now);
        let schedule = serde_json::to_string(&scheduling_cards).unwrap();
        println!("{}", schedule);
    }
}
