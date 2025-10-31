//! DateTime-related generators (std::time only)
//!
//! Provides generators for:
//! - Unix timestamps (i64)
//! - Duration values
//! - SystemTime ranges
//!
//! All generators use std library only (no chrono dependency).

use protest::{Generator, GeneratorConfig};
use rand::Rng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// Timestamp Generator
// ============================================================================

/// Generator for Unix timestamps (seconds since epoch)
///
/// Generates i64 timestamps that can be converted to SystemTime or used with datetime libraries
#[derive(Debug, Clone)]
pub struct TimestampGenerator {
    min: i64,
    max: i64,
}

impl TimestampGenerator {
    /// Create a new timestamp generator with bounds
    pub fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }

    /// Create a timestamp generator for recent dates (last 10 years to now)
    pub fn recent() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let ten_years_ago = now - (10 * 365 * 24 * 60 * 60);
        Self {
            min: ten_years_ago,
            max: now,
        }
    }

    /// Create a timestamp generator for future dates (now to 10 years from now)
    pub fn future() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let ten_years_later = now + (10 * 365 * 24 * 60 * 60);
        Self {
            min: now,
            max: ten_years_later,
        }
    }
}

impl Generator<i64> for TimestampGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> i64 {
        rng.r#gen_range(self.min..=self.max)
    }

    fn shrink(&self, value: &i64) -> Box<dyn Iterator<Item = i64>> {
        let mut shrinks = Vec::new();

        // Try epoch (0)
        if *value > 0 && self.min <= 0 {
            shrinks.push(0);
        }

        // Try min
        if *value > self.min {
            shrinks.push(self.min);
        }

        // Try half
        if *value > self.min {
            let half = (*value + self.min) / 2;
            if half >= self.min {
                shrinks.push(half);
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Duration Generator
// ============================================================================

/// Generator for Duration values
#[derive(Debug, Clone)]
pub struct DurationGenerator {
    min_secs: u64,
    max_secs: u64,
}

impl DurationGenerator {
    /// Create a new duration generator with bounds in seconds
    pub fn new(min_secs: u64, max_secs: u64) -> Self {
        Self { min_secs, max_secs }
    }

    /// Create a duration generator for milliseconds (0-1000ms)
    pub fn milliseconds() -> Self {
        Self {
            min_secs: 0,
            max_secs: 1,
        }
    }

    /// Create a duration generator for seconds (0-60s)
    pub fn seconds() -> Self {
        Self {
            min_secs: 0,
            max_secs: 60,
        }
    }

    /// Create a duration generator for minutes (0-60 minutes)
    pub fn minutes() -> Self {
        Self {
            min_secs: 0,
            max_secs: 60 * 60,
        }
    }

    /// Create a duration generator for hours (0-24 hours)
    pub fn hours() -> Self {
        Self {
            min_secs: 0,
            max_secs: 24 * 60 * 60,
        }
    }
}

impl Generator<Duration> for DurationGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> Duration {
        let secs = rng.r#gen_range(self.min_secs..=self.max_secs);
        let nanos = rng.r#gen_range(0..1_000_000_000);
        Duration::new(secs, nanos)
    }

    fn shrink(&self, value: &Duration) -> Box<dyn Iterator<Item = Duration>> {
        let mut shrinks = Vec::new();

        // Try zero
        if value.as_secs() > 0 || value.subsec_nanos() > 0 {
            shrinks.push(Duration::ZERO);
        }

        // Try min duration
        let min_dur = Duration::from_secs(self.min_secs);
        if *value > min_dur {
            shrinks.push(min_dur);
        }

        // Try half the duration
        if value.as_secs() > 0 {
            shrinks.push(Duration::from_secs(value.as_secs() / 2));
        }

        // Try removing subsec nanos
        if value.subsec_nanos() > 0 {
            shrinks.push(Duration::from_secs(value.as_secs()));
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// SystemTime Generator
// ============================================================================

/// Generator for SystemTime values
#[derive(Debug, Clone)]
pub struct SystemTimeGenerator {
    start: SystemTime,
    max_offset_secs: u64,
}

impl SystemTimeGenerator {
    /// Create a new SystemTime generator
    ///
    /// Generates times from `start` to `start + max_offset_secs`
    pub fn new(start: SystemTime, max_offset_secs: u64) -> Self {
        Self {
            start,
            max_offset_secs,
        }
    }

    /// Create a generator for times around now (± 1 year)
    pub fn around_now() -> Self {
        let now = SystemTime::now();
        let one_year = 365 * 24 * 60 * 60;

        // Start from 1 year ago
        let start = now - Duration::from_secs(one_year);

        Self {
            start,
            max_offset_secs: one_year * 2, // ±1 year = 2 years total
        }
    }

    /// Create a generator starting from Unix epoch
    pub fn from_epoch(max_offset_secs: u64) -> Self {
        Self {
            start: UNIX_EPOCH,
            max_offset_secs,
        }
    }
}

impl Generator<SystemTime> for SystemTimeGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> SystemTime {
        let offset_secs = rng.r#gen_range(0..=self.max_offset_secs);
        self.start + Duration::from_secs(offset_secs)
    }

    fn shrink(&self, value: &SystemTime) -> Box<dyn Iterator<Item = SystemTime>> {
        let mut shrinks = Vec::new();

        // Try start time
        if *value > self.start {
            shrinks.push(self.start);
        }

        // Try halfway between start and value
        if let Ok(duration_since_start) = value.duration_since(self.start) {
            let half_secs = duration_since_start.as_secs() / 2;
            if half_secs > 0 {
                shrinks.push(self.start + Duration::from_secs(half_secs));
            }
        }

        Box::new(shrinks.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_timestamp_generator() {
        let gen = TimestampGenerator::new(0, 1_000_000);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let ts = gen.generate(&mut rng, &config);
            assert!(ts >= 0 && ts <= 1_000_000);
        }
    }

    #[test]
    fn test_timestamp_recent() {
        let gen = TimestampGenerator::recent();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for _ in 0..10 {
            let ts = gen.generate(&mut rng, &config);
            assert!(ts <= now);
            assert!(ts >= now - (10 * 365 * 24 * 60 * 60));
        }
    }

    #[test]
    fn test_duration_generator() {
        let gen = DurationGenerator::new(0, 100);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let dur = gen.generate(&mut rng, &config);
            assert!(dur.as_secs() <= 100);
        }
    }

    #[test]
    fn test_duration_presets() {
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let gen = DurationGenerator::seconds();
        let dur = gen.generate(&mut rng, &config);
        assert!(dur.as_secs() <= 60);

        let gen = DurationGenerator::minutes();
        let dur = gen.generate(&mut rng, &config);
        assert!(dur.as_secs() <= 3600);
    }

    #[test]
    fn test_system_time_generator() {
        let start = UNIX_EPOCH;
        let gen = SystemTimeGenerator::new(start, 1_000_000);
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let time = gen.generate(&mut rng, &config);
            assert!(time >= start);

            let duration = time.duration_since(start).unwrap();
            assert!(duration.as_secs() <= 1_000_000);
        }
    }

    #[test]
    fn test_system_time_around_now() {
        let gen = SystemTimeGenerator::around_now();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        let now = SystemTime::now();

        for _ in 0..10 {
            let time = gen.generate(&mut rng, &config);

            // Should be within reasonable range of now
            // (allowing for test execution time)
            if let Ok(duration) = now.duration_since(time) {
                assert!(duration.as_secs() <= 365 * 24 * 60 * 60 + 10);
            } else if let Ok(duration) = time.duration_since(now) {
                assert!(duration.as_secs() <= 365 * 24 * 60 * 60 + 10);
            }
        }
    }
}
