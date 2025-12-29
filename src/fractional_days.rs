use chrono::Duration;

/// Trait for working with fractional days in chrono::Duration
pub trait FractionalDays {
    /// Convert Duration to fractional days (e.g., 1.5 days, 0.5 days)
    fn num_fractional_days(&self) -> f64;

    /// Create Duration from fractional days
    fn fractional_days(fractional_days: f64) -> Self;
}

impl FractionalDays for Duration {
    fn num_fractional_days(&self) -> f64 {
        const SECONDS_IN_A_DAY: f64 = 24.0 * 3600.0;
        self.num_seconds() as f64 / SECONDS_IN_A_DAY
    }

    fn fractional_days(fractional_days: f64) -> Self {
        Self::seconds((fractional_days * 24.0 * 60.0 * 60.0).round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_num_fractional_days() {
        // Test whole days
        let duration = Duration::days(1);
        assert_eq!(duration.num_fractional_days(), 1.0);

        let duration = Duration::days(2);
        assert_eq!(duration.num_fractional_days(), 2.0);

        // Test fractional days
        let duration = Duration::hours(36); // 1.5 days
        assert_eq!(duration.num_fractional_days(), 1.5);

        let duration = Duration::hours(12); // 0.5 days
        assert_eq!(duration.num_fractional_days(), 0.5);

        // Test zero
        let duration = Duration::zero();
        assert_eq!(duration.num_fractional_days(), 0.0);
    }

    #[test]
    fn test_fractional_days() {
        // Test whole days
        let duration = Duration::fractional_days(1.0);
        assert_eq!(duration.num_seconds(), 86400);

        let duration = Duration::fractional_days(2.0);
        assert_eq!(duration.num_seconds(), 172800);

        // Test fractional days
        let duration = Duration::fractional_days(1.5);
        assert_eq!(duration.num_seconds(), 129600);

        let duration = Duration::fractional_days(0.5);
        assert_eq!(duration.num_seconds(), 43200);

        // Test zero
        let duration = Duration::fractional_days(0.0);
        assert_eq!(duration.num_seconds(), 0);
    }

    #[test]
    fn test_roundtrip() {
        // Test that converting to fractional days and back preserves the value
        let original = 1.5;
        let duration = Duration::fractional_days(original);
        let result = duration.num_fractional_days();
        assert!((result - original).abs() < 1e-10);

        let original = 7.25;
        let duration = Duration::fractional_days(original);
        let result = duration.num_fractional_days();
        assert!((result - original).abs() < 1e-10);
    }
}
