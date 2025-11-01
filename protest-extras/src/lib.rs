#![allow(clippy::module_inception)]

//! # Protest Extras
//!
//! Additional generators for the Protest property testing library.
//!
//! This crate provides extra generators for:
//! - **Network**: IP addresses, URLs, email addresses
//! - **DateTime**: Unix timestamps, durations, system time ranges
//! - **Text**: Alphabetic, alphanumeric, identifiers, sentences
//! - **Collections**: Non-empty, sorted, unique collections
//! - **Numeric**: Positive integers, even numbers, primes, percentages
//! - **Domain**: UUIDs, Base64, hex strings, file paths
//!
//! All features use **std library only** by default, with optional external dependencies
//! available via feature flags.
//!
//! ## No External Dependencies
//!
//! This crate uses **std library only** (plus `rand` which you already have from `protest`).
//! All generators including UUID v4 are implemented without external dependencies.
//!
//! ## Quick Start
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::Generator;
//! use rand::thread_rng;
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate an IPv4 address
//! let ip_gen = IpAddressGenerator::ipv4();
//! let ip = ip_gen.generate(&mut rng, &config);
//! assert!(ip.contains('.'));
//!
//! // Generate an email address
//! let email_gen = EmailGenerator::new();
//! let email = email_gen.generate(&mut rng, &config);
//! assert!(email.contains('@'));
//!
//! // Generate a UUID
//! let uuid_gen = UuidV4Generator::new();
//! let uuid = uuid_gen.generate(&mut rng, &config);
//! assert_eq!(uuid.len(), 36); // UUID format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
//! # }
//! ```
//!
//! ## Examples by Category
//!
//! ### Network Generators
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::Generator;
//! use rand::thread_rng;
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate IPv4 addresses
//! let generator = IpAddressGenerator::ipv4();
//! let ip = generator.generate(&mut rng, &config);
//! let parts: Vec<&str> = ip.split('.').collect();
//! assert_eq!(parts.len(), 4);
//!
//! // Generate IPv6 addresses
//! let generator = IpAddressGenerator::ipv6();
//! let ip = generator.generate(&mut rng, &config);
//! assert!(ip.contains(':'));
//!
//! // Generate email addresses
//! let generator = EmailGenerator::new();
//! let email = generator.generate(&mut rng, &config);
//! assert_eq!(email.matches('@').count(), 1);
//!
//! // Generate URLs
//! let generator = UrlGenerator::new();
//! let url = generator.generate(&mut rng, &config);
//! assert!(url.starts_with("http://") || url.starts_with("https://"));
//! # }
//! ```
//!
//! ### DateTime Generators
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::Generator;
//! use rand::thread_rng;
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate recent Unix timestamps
//! let generator = TimestampGenerator::recent();
//! let timestamp = generator.generate(&mut rng, &config);
//! let now = SystemTime::now()
//!     .duration_since(UNIX_EPOCH)
//!     .unwrap()
//!     .as_secs() as i64;
//! assert!(timestamp <= now);
//!
//! // Generate durations
//! let generator = DurationGenerator::seconds();
//! let duration = generator.generate(&mut rng, &config);
//! assert!(duration.as_secs() <= 60);
//!
//! // Generate system times around now
//! let generator = SystemTimeGenerator::around_now();
//! let time = generator.generate(&mut rng, &config);
//! // Time should be valid
//! assert!(time.duration_since(UNIX_EPOCH).is_ok());
//! # }
//! ```
//!
//! ### Text Generators
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::Generator;
//! use rand::thread_rng;
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate lowercase alphabetic strings
//! let generator = AlphabeticGenerator::lowercase(5, 10);
//! let text = generator.generate(&mut rng, &config);
//! assert!(text.len() >= 5 && text.len() <= 10);
//! assert!(text.chars().all(|c| c.is_lowercase()));
//!
//! // Generate valid identifiers
//! let generator = IdentifierGenerator::new(3, 15);
//! let id = generator.generate(&mut rng, &config);
//! assert!(id.chars().next().unwrap().is_alphabetic() || id.chars().next().unwrap() == '_');
//! assert!(id.chars().all(|c| c.is_alphanumeric() || c == '_'));
//!
//! // Generate sentences
//! let generator = SentenceGenerator::new(3, 10);
//! let sentence = generator.generate(&mut rng, &config);
//! assert!(sentence.ends_with('.'));
//! # }
//! ```
//!
//! ### Collection Generators
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::{Generator, IntGenerator};
//! use rand::thread_rng;
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate non-empty vectors
//! let generator = NonEmptyVecGenerator::new(IntGenerator::new(0, 100), 1, 10);
//! let vec = generator.generate(&mut rng, &config);
//! assert!(!vec.is_empty());
//! assert!(vec.len() <= 10);
//!
//! // Generate sorted vectors
//! let generator = SortedVecGenerator::new(IntGenerator::new(0, 100), 0, 20);
//! let vec = generator.generate(&mut rng, &config);
//! for window in vec.windows(2) {
//!     assert!(window[0] <= window[1]);
//! }
//!
//! // Generate vectors with unique elements
//! let generator = UniqueVecGenerator::new(IntGenerator::new(0, 1000), 5, 20);
//! let vec = generator.generate(&mut rng, &config);
//! let unique_count = vec.iter().collect::<std::collections::HashSet<_>>().len();
//! assert_eq!(unique_count, vec.len());
//! # }
//! ```
//!
//! ### Numeric Generators
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::Generator;
//! use rand::thread_rng;
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate positive integers
//! let generator = PositiveIntGenerator::<u32>::new(1, 1000);
//! let n = generator.generate(&mut rng, &config);
//! assert!(n >= 1 && n <= 1000);
//!
//! // Generate even numbers
//! let generator = EvenNumberGenerator::new(0i32, 100i32);
//! let n = generator.generate(&mut rng, &config);
//! assert_eq!(n % 2, 0);
//!
//! // Generate prime numbers
//! let generator = PrimeNumberGenerator::new(2, 100);
//! let n = generator.generate(&mut rng, &config);
//! assert!(n >= 2);
//!
//! // Generate percentages
//! let generator = PercentageGenerator::new();
//! let p = generator.generate(&mut rng, &config);
//! assert!(p >= 0.0 && p <= 100.0);
//! # }
//! ```
//!
//! ### Domain Generators
//!
//! ```rust
//! use protest_extras::prelude::*;
//! use protest::Generator;
//! use rand::thread_rng;
//!
//! # fn main() {
//! let mut rng = thread_rng();
//! let config = protest::GeneratorConfig::default();
//!
//! // Generate hexadecimal strings
//! let generator = HexGenerator::new(8, 16);
//! let hex = generator.generate(&mut rng, &config);
//! assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
//!
//! // Generate Base64 strings
//! let generator = Base64Generator::new(6, 32);
//! let b64 = generator.generate(&mut rng, &config);
//! assert!(!b64.is_empty());
//!
//! // Generate file paths
//! let generator = PathGenerator::new(1, 4);
//! let path = generator.generate(&mut rng, &config);
//! assert!(path.components().count() >= 1);
//!
//! // Generate UUID v4
//! let generator = UuidV4Generator::new();
//! let uuid = generator.generate(&mut rng, &config);
//! let parts: Vec<&str> = uuid.split('-').collect();
//! assert_eq!(parts.len(), 5);
//! assert_eq!(parts[2].chars().next().unwrap(), '4'); // Version 4
//! # }
//! ```

// Re-export protest for convenience
pub use protest;

// Generators module
pub mod generators;

// Shrinking strategies module
pub mod shrinking;

// Re-export commonly used items
pub mod prelude {
    //! Convenient re-exports of commonly used generators

    // Network generators
    pub use crate::generators::network::{EmailGenerator, IpAddressGenerator, UrlGenerator};

    // DateTime generators
    pub use crate::generators::datetime::{
        DurationGenerator, SystemTimeGenerator, TimestampGenerator,
    };

    // Text generators
    pub use crate::generators::text::{
        AlphabeticGenerator, AlphanumericGenerator, IdentifierGenerator, ParagraphGenerator,
        SentenceGenerator,
    };

    // Collection generators
    pub use crate::generators::collections::{
        BoundedMapGenerator, NonEmptyVecGenerator, SortedVecGenerator, UniqueVecGenerator,
    };

    // Numeric generators
    pub use crate::generators::numeric::{
        EvenNumberGenerator, PercentageGenerator, PositiveIntGenerator, PrimeNumberGenerator,
    };

    // Domain generators
    pub use crate::generators::domain::{
        Base64Generator, HexGenerator, PathGenerator, UuidV4Generator,
    };

    // Shrinking strategies
    pub use crate::shrinking::{
        CascadingShrinker, ConfigurableShrinker, DeltaDebugShrinker, GuidedShrinker,
        ShrinkStrategy, SmartShrink, TargetedShrinker,
    };
}
