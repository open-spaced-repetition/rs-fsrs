use crate::Rating;

type Weights = [f64; 19];
const DEFAULT_WEIGHTS: Weights = [
    0.4197, 1.1869, 3.0412, 15.2441, 7.1434, 0.6477, 1.0007, 0.0674, 1.6597, 0.1712, 1.1178,
    2.0225, 0.0904, 0.3025, 2.1214, 0.2498, 2.9466, 0.4891, 0.6468,
];

#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    pub request_retention: f64,
    pub maximum_interval: i32,
    pub w: Weights,
    pub decay: f64,
    pub factor: f64,
    pub enable_short_term: bool,
}

impl Parameters {
    pub const DECAY: f64 = -0.5;
    /// (9/10) ^ (1 / DECAY) - 1
    pub const FACTOR: f64 = 19f64 / 81f64;

    pub fn forgeting_curve(&self, elapsed_days: i64, stability: f64) -> f64 {
        (1.0 + Self::FACTOR * elapsed_days as f64 / stability).powf(Self::DECAY)
    }

    pub fn init_difficulty(&self, rating: Rating) -> f64 {
        let rating_int: i32 = rating as i32;

        (self.w[4] - f64::exp(self.w[5] * (rating_int as f64 - 1.0)) + 1.0).clamp(1.0, 10.0)
    }

    pub fn init_stability(&self, rating: Rating) -> f64 {
        let rating_int: i32 = rating as i32;
        self.w[(rating_int - 1) as usize].max(0.1)
    }

    #[allow(clippy::suboptimal_flops)]
    pub fn next_interval(&self, stability: f64) -> i64 {
        let new_interval =
            stability / Self::FACTOR * (self.request_retention.powf(1.0 / Self::DECAY) - 1.0);
        (new_interval.round() as i64).clamp(1, self.maximum_interval as i64)
    }

    pub fn next_difficulty(&self, difficulty: f64, rating: Rating) -> f64 {
        let rating_int = rating as i32;
        let next_difficulty = self.w[6].mul_add(-(rating_int as f64 - 3.0), difficulty);
        let mean_reversion =
            self.mean_reversion(self.init_difficulty(Rating::Easy), next_difficulty);
        mean_reversion.clamp(1.0, 10.0)
    }

    pub fn short_term_stability(&self, stability: f64, rating: Rating) -> f64 {
        let rating_int = rating as i32;
        stability * f64::exp(self.w[17] * (rating_int as f64 - 3.0 + self.w[18]))
    }

    pub fn next_recall_stability(
        &self,
        difficulty: f64,
        stability: f64,
        retrievability: f64,
        rating: Rating,
    ) -> f64 {
        let modifier = match rating {
            Rating::Hard => self.w[15],
            Rating::Easy => self.w[16],
            _ => 1.0,
        };

        stability
            * (((self.w[8]).exp()
                * (11.0 - difficulty)
                * stability.powf(-self.w[9])
                * (((1.0 - retrievability) * self.w[10]).exp_m1()))
            .mul_add(modifier, 1.0))
    }

    pub fn next_forget_stability(
        &self,
        difficulty: f64,
        stability: f64,
        retrievability: f64,
    ) -> f64 {
        self.w[11]
            * difficulty.powf(-self.w[12])
            * ((stability + 1.0).powf(self.w[13]) - 1.0)
            * f64::exp((1.0 - retrievability) * self.w[14])
    }

    fn mean_reversion(&self, initial: f64, current: f64) -> f64 {
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
