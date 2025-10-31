//! RNG backend integration and provider system.

use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::{Arc, Mutex};

/// Trait for providing random number generators
pub trait RngProvider: Send + Sync {
    /// The type of RNG this provider creates
    type Rng: rand::RngCore + Clone + Send;

    /// Create a new RNG instance with an optional seed
    fn create_rng(&self, seed: Option<u64>) -> Self::Rng;

    /// Create a new RNG instance with a random seed
    fn create_random_rng(&self) -> Self::Rng {
        self.create_rng(None)
    }
}

/// Default RNG provider using the standard library's StdRng
#[derive(Debug, Clone)]
pub struct DefaultRngProvider;

impl RngProvider for DefaultRngProvider {
    type Rng = StdRng;

    fn create_rng(&self, seed: Option<u64>) -> Self::Rng {
        match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        }
    }
}

/// Thread-safe RNG state manager
#[derive(Debug)]
pub struct RngManager<P: RngProvider> {
    provider: P,
    seed: Option<u64>,
    // Thread-local RNG cache for performance
    thread_rng: Arc<Mutex<Option<P::Rng>>>,
}

impl<P: RngProvider> RngManager<P> {
    /// Create a new RNG manager with the given provider
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            seed: None,
            thread_rng: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new RNG manager with a specific seed
    pub fn with_seed(provider: P, seed: u64) -> Self {
        Self {
            provider,
            seed: Some(seed),
            thread_rng: Arc::new(Mutex::new(None)),
        }
    }

    /// Get or create an RNG instance for the current thread
    pub fn get_rng(&self) -> P::Rng {
        // For simplicity, always create a new RNG
        // In a real implementation, we might want thread-local storage
        self.provider.create_rng(self.seed)
    }

    /// Create a new RNG with a specific seed, regardless of the manager's seed
    pub fn create_seeded_rng(&self, seed: u64) -> P::Rng {
        self.provider.create_rng(Some(seed))
    }

    /// Get the current seed, if any
    pub fn seed(&self) -> Option<u64> {
        self.seed
    }

    /// Set a new seed for future RNG instances
    pub fn set_seed(&mut self, seed: Option<u64>) {
        self.seed = seed;
        // Clear the cached RNG so it gets recreated with the new seed
        if let Ok(mut cached) = self.thread_rng.lock() {
            *cached = None;
        }
    }
}

impl<P: RngProvider + Clone> Clone for RngManager<P> {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            seed: self.seed,
            thread_rng: Arc::new(Mutex::new(None)),
        }
    }
}

use std::sync::OnceLock;

/// Global RNG manager instance
static GLOBAL_RNG_MANAGER: OnceLock<RngManager<DefaultRngProvider>> = OnceLock::new();

/// Get the global RNG manager, initializing it if necessary
pub fn global_rng_manager() -> &'static RngManager<DefaultRngProvider> {
    GLOBAL_RNG_MANAGER.get_or_init(|| RngManager::new(DefaultRngProvider))
}

/// Set the global seed for reproducible testing
/// Note: This is a simplified implementation. In practice, you'd want to call this
/// before any other RNG operations to ensure consistency.
pub fn set_global_seed(_seed: u64) {
    // This is a placeholder - in a real implementation, we'd need a more
    // sophisticated approach to handle seed setting after initialization
    // For now, users should create their own RngManager with a seed
}

/// Create a new RNG from the global manager
pub fn create_rng() -> StdRng {
    global_rng_manager().get_rng()
}

/// Create a new RNG with a specific seed
pub fn create_seeded_rng(seed: u64) -> StdRng {
    global_rng_manager().create_seeded_rng(seed)
}

/// A custom RNG provider for testing purposes
#[derive(Debug, Clone)]
pub struct TestRngProvider {
    base_seed: u64,
}

impl TestRngProvider {
    /// Create a new test RNG provider with a base seed
    pub fn new(base_seed: u64) -> Self {
        Self { base_seed }
    }
}

impl RngProvider for TestRngProvider {
    type Rng = StdRng;

    fn create_rng(&self, seed: Option<u64>) -> Self::Rng {
        let actual_seed = seed.unwrap_or(self.base_seed);
        StdRng::seed_from_u64(actual_seed)
    }
}

/// Wrapper to make any RNG work with our generator system
#[derive(Debug)]
pub struct RngWrapper<R> {
    inner: R,
}

impl<R> RngWrapper<R> {
    /// Create a new RNG wrapper
    pub fn new(rng: R) -> Self {
        Self { inner: rng }
    }

    /// Get a mutable reference to the inner RNG
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Get a reference to the inner RNG
    pub fn inner(&self) -> &R {
        &self.inner
    }
}

impl<R: rand::RngCore> rand::RngCore for RngWrapper<R> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.inner.try_fill_bytes(dest)
    }
}

impl<R: rand::RngCore + Clone> Clone for RngWrapper<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, RngCore};

    #[test]
    fn test_default_rng_provider() {
        let provider = DefaultRngProvider;

        // Test creating RNG without seed
        let mut rng1 = provider.create_rng(None);
        let mut rng2 = provider.create_rng(None);

        // They should produce different values (very likely)
        let _val1: u32 = rng1.r#gen();
        let _val2: u32 = rng2.r#gen();
        // Note: There's a tiny chance they could be equal, but it's very unlikely

        // Test creating RNG with seed
        let mut rng3 = provider.create_rng(Some(12345));
        let mut rng4 = provider.create_rng(Some(12345));

        // They should produce the same values
        let val3: u32 = rng3.r#gen();
        let val4: u32 = rng4.r#gen();
        assert_eq!(val3, val4);
    }

    #[test]
    fn test_rng_manager() {
        let provider = DefaultRngProvider;
        let manager = RngManager::new(provider);

        // Test getting RNG
        let mut rng = manager.get_rng();
        let _value: u32 = rng.r#gen();

        // Test seeded RNG
        let mut seeded_rng = manager.create_seeded_rng(42);
        let seeded_value: u32 = seeded_rng.r#gen();

        // Create another RNG with the same seed
        let mut seeded_rng2 = manager.create_seeded_rng(42);
        let seeded_value2: u32 = seeded_rng2.r#gen();

        assert_eq!(seeded_value, seeded_value2);
    }

    #[test]
    fn test_rng_manager_with_seed() {
        let provider = DefaultRngProvider;
        let manager = RngManager::with_seed(provider, 999);

        assert_eq!(manager.seed(), Some(999));

        // Test that RNGs created from this manager use the seed
        let mut rng1 = manager.get_rng();
        let mut rng2 = manager.get_rng();

        let val1: u32 = rng1.r#gen();
        let val2: u32 = rng2.r#gen();

        // They should be the same since they use the same seed
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_test_rng_provider() {
        let provider = TestRngProvider::new(777);

        let mut rng1 = provider.create_rng(None);
        let mut rng2 = provider.create_rng(None);

        let val1: u32 = rng1.r#gen();
        let val2: u32 = rng2.r#gen();

        // Should be the same since they use the same base seed
        assert_eq!(val1, val2);

        // Test with explicit seed
        let mut rng3 = provider.create_rng(Some(888));
        let val3: u32 = rng3.r#gen();

        // Should be different from the base seed values
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_rng_wrapper() {
        let base_rng = StdRng::seed_from_u64(12345);
        let mut wrapper = RngWrapper::new(base_rng);

        // Test that it works as an RNG
        let _value = wrapper.next_u32();
        let _value = wrapper.next_u64();

        let mut bytes = [0u8; 10];
        wrapper.fill_bytes(&mut bytes);

        // Test that we can access the inner RNG
        let _inner = wrapper.inner();
        let _inner_mut = wrapper.inner_mut();
    }

    #[test]
    fn test_global_rng_functions() {
        // Test creating RNG from global manager
        let mut rng = create_rng();
        let _value: u32 = rng.r#gen();

        // Test creating seeded RNG
        let mut seeded_rng1 = create_seeded_rng(555);
        let mut seeded_rng2 = create_seeded_rng(555);

        let val1: u32 = seeded_rng1.r#gen();
        let val2: u32 = seeded_rng2.r#gen();

        assert_eq!(val1, val2);
    }

    #[test]
    fn test_rng_manager_clone() {
        let provider = DefaultRngProvider;
        let manager1 = RngManager::with_seed(provider, 123);
        let manager2 = manager1.clone();

        assert_eq!(manager1.seed(), manager2.seed());

        let mut rng1 = manager1.get_rng();
        let mut rng2 = manager2.get_rng();

        let val1: u32 = rng1.r#gen();
        let val2: u32 = rng2.r#gen();

        // Should be the same since they have the same seed
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_rng_manager_set_seed() {
        let provider = DefaultRngProvider;
        let mut manager = RngManager::new(provider);

        assert_eq!(manager.seed(), None);

        manager.set_seed(Some(456));
        assert_eq!(manager.seed(), Some(456));

        manager.set_seed(None);
        assert_eq!(manager.seed(), None);
    }
}
