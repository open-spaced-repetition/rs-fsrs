use crate::Rating::{Again, Easy, Hard};
use crate::State::{Learning, Relearning, Review};
use crate::{Card, Rating, ScheduledCards, State};

type Weights = [f32; 19];
const DEFAULT_WEIGHTS: Weights = [
    0.4197, 1.1869, 3.0412, 15.2441, 7.1434, 0.6477, 1.0007, 0.0674, 1.6597, 0.1712, 1.1178,
    2.0225, 0.0904, 0.3025, 2.1214, 0.2498, 2.9466, 0.4891, 0.6468,
];

#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    pub request_retention: f32,
    pub maximum_interval: i32,
    pub w: Weights,
    pub decay: f32,
    pub factor: f32,
    pub enable_short_term: bool,
}

impl Parameters {
    pub const DECAY: f32 = -0.5;
    /// (9/10) ^ (1 / DECAY) - 1
    pub const FACTOR: f32 = 19f32 / 81f32;

    pub fn forgeting_curve(card: &Card) -> f32 {
        (1.0 + Self::FACTOR * card.elapsed_days as f32 / card.stability).powf(Self::DECAY)
    }

    pub fn init_difficulty(&self, rating: Rating) -> f32 {
        let rating_int: i32 = rating as i32;

        (self.w[4] - f32::exp(self.w[5] * (rating_int as f32 - 1.0)) + 1.0).clamp(1.0, 10.0)
    }

    pub fn init_stability(&self, rating: Rating) -> f32 {
        let rating_int: i32 = rating as i32;
        self.w[(rating_int - 1) as usize].max(0.1)
    }

    pub fn init_difficulty_stability(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            let Some(card) = output_cards.cards.get_mut(rating) else {
                continue;
            };
            card.difficulty = self.init_difficulty(*rating);
            card.stability = self.init_stability(*rating);
        }
    }

    #[allow(clippy::suboptimal_flops)]
    pub fn next_interval(
        &self,
        output_cards: &mut ScheduledCards,
        rating: Rating,
    ) -> Result<i64, String> {
        let Some(card) = output_cards.cards.get_mut(&rating) else {
            return Err("Failed to retrieve card from output_cards".to_string());
        };
        let new_interval =
            card.stability / Self::FACTOR * (self.request_retention.powf(1.0 / Self::DECAY) - 1.0);
        Ok((new_interval.round() as i64).clamp(1, self.maximum_interval as i64))
    }

    pub fn next_difficulty(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            let rating_int = *rating as i32;
            let Some(mut card) = output_cards.cards.remove(rating) else {
                continue;
            };
            let next_difficulty = self.w[6].mul_add(-(rating_int as f32 - 3.0), card.difficulty);
            let mean_reversion = self.mean_reversion(self.init_difficulty(Easy), next_difficulty);
            card.difficulty = mean_reversion.clamp(1.0, 10.0);
            output_cards.cards.insert(*rating, card);
        }
    }

    pub fn next_stability(&self, output_cards: &mut ScheduledCards, state: State) {
        if state == Learning || state == Relearning {
            self.short_term_stability(output_cards)
        } else if state == Review {
            for rating in Rating::iter() {
                if rating == &Again {
                    self.next_forget_stability(output_cards);
                } else {
                    self.next_recall_stability(output_cards, *rating);
                }
            }
        }
    }

    fn short_term_stability(&self, output_cards: &mut ScheduledCards) {
        for rating in Rating::iter() {
            let rating_int = *rating as i32;
            let Some(card) = output_cards.cards.get_mut(rating) else {
                continue;
            };
            card.stability *= f32::exp(self.w[17] * (rating_int as f32 - 3.0 + self.w[18]));
        }
    }

    fn next_recall_stability(&self, output_cards: &mut ScheduledCards, rating: Rating) {
        let modifier = match rating {
            Hard => self.w[15],
            Easy => self.w[16],
            _ => 1.0,
        };

        let Some(card) = output_cards.cards.get_mut(&rating) else {
            return;
        };
        let retrievability = card.get_retrievability();
        card.stability = card.stability
            * (((self.w[8]).exp()
                * (11.0 - card.difficulty)
                * card.stability.powf(-self.w[9])
                * (((1.0 - retrievability) * self.w[10]).exp_m1()))
            .mul_add(modifier, 1.0));
    }

    fn next_forget_stability(&self, output_cards: &mut ScheduledCards) {
        let Some(card) = output_cards.cards.get_mut(&Again) else {
            return;
        };
        let retrievability = card.get_retrievability();
        card.stability = self.w[11]
            * card.difficulty.powf(-self.w[12])
            * ((card.stability + 1.0).powf(self.w[13]) - 1.0)
            * f32::exp((1.0 - retrievability) * self.w[14])
    }

    fn mean_reversion(&self, initial: f32, current: f32) -> f32 {
        self.w[7].mul_add(initial, (1.0 - self.w[7]) * current)
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            request_retention: 0.9,
            maximum_interval: 36500,
            w: DEFAULT_WEIGHTS,
            decay: Self::DECAY,
            factor: Self::FACTOR,
            enable_short_term: true,
        }
    }
}
