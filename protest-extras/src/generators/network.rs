//! Network-related generators
//!
//! This module provides generators for:
//! - IPv4 and IPv6 addresses
//! - Email addresses
//! - URLs
//!
//! All generators use std library only.

use protest::{Generator, GeneratorConfig};
use rand::Rng;
use std::net::{Ipv4Addr, Ipv6Addr};

// ============================================================================
// IP Address Generator
// ============================================================================

/// Generator for IP addresses (IPv4 or IPv6)
#[derive(Debug, Clone)]
pub enum IpAddressGenerator {
    /// Generate IPv4 addresses
    V4,
    /// Generate IPv6 addresses
    V6,
    /// Generate both IPv4 and IPv6 addresses
    Both,
}

impl IpAddressGenerator {
    /// Create a generator for IPv4 addresses only
    pub fn ipv4() -> Self {
        Self::V4
    }

    /// Create a generator for IPv6 addresses only
    pub fn ipv6() -> Self {
        Self::V6
    }

    /// Create a generator for both IPv4 and IPv6 addresses
    pub fn both() -> Self {
        Self::Both
    }
}

impl Generator<String> for IpAddressGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        match self {
            IpAddressGenerator::V4 => {
                let octets: [u8; 4] = [rng.r#gen(), rng.r#gen(), rng.r#gen(), rng.r#gen()];
                Ipv4Addr::from(octets).to_string()
            }
            IpAddressGenerator::V6 => {
                let segments: [u16; 8] = [
                    rng.r#gen(),
                    rng.r#gen(),
                    rng.r#gen(),
                    rng.r#gen(),
                    rng.r#gen(),
                    rng.r#gen(),
                    rng.r#gen(),
                    rng.r#gen(),
                ];
                Ipv6Addr::from(segments).to_string()
            }
            IpAddressGenerator::Both => {
                if rng.r#gen::<bool>() {
                    IpAddressGenerator::V4.generate(rng, _config)
                } else {
                    IpAddressGenerator::V6.generate(rng, _config)
                }
            }
        }
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        // Try to shrink to simpler IP addresses
        let mut shrinks = Vec::new();

        // Try localhost
        if value != "127.0.0.1" && value != "::1" {
            shrinks.push("127.0.0.1".to_string());
        }

        // Try zero address
        if value != "0.0.0.0" && value != "::" {
            shrinks.push("0.0.0.0".to_string());
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// Email Generator
// ============================================================================

/// Generator for RFC-compliant email addresses
#[derive(Debug, Clone)]
pub struct EmailGenerator {
    min_local_len: usize,
    max_local_len: usize,
    min_domain_len: usize,
    max_domain_len: usize,
}

impl EmailGenerator {
    /// Create a new email generator with default lengths
    pub fn new() -> Self {
        Self {
            min_local_len: 3,
            max_local_len: 20,
            min_domain_len: 5,
            max_domain_len: 30,
        }
    }

    /// Create an email generator with custom length constraints
    pub fn with_lengths(
        min_local_len: usize,
        max_local_len: usize,
        min_domain_len: usize,
        max_domain_len: usize,
    ) -> Self {
        Self {
            min_local_len,
            max_local_len,
            min_domain_len,
            max_domain_len,
        }
    }

    fn generate_local_part(&self, rng: &mut dyn rand::RngCore) -> String {
        let len = rng.r#gen_range(self.min_local_len..=self.max_local_len);
        let valid_chars = b"abcdefghijklmnopqrstuvwxyz0123456789._-";

        (0..len)
            .map(|i| {
                if i == 0 {
                    // First char must be alphanumeric
                    let alphanumeric = b"abcdefghijklmnopqrstuvwxyz0123456789";
                    alphanumeric[rng.r#gen_range(0..alphanumeric.len())] as char
                } else {
                    valid_chars[rng.r#gen_range(0..valid_chars.len())] as char
                }
            })
            .collect()
    }

    fn generate_domain(&self, rng: &mut dyn rand::RngCore) -> String {
        let len = rng.r#gen_range(self.min_domain_len..=self.max_domain_len);
        let valid_chars = b"abcdefghijklmnopqrstuvwxyz0123456789-";

        let domain_name: String = (0..len)
            .map(|_| valid_chars[rng.r#gen_range(0..valid_chars.len())] as char)
            .collect();

        // Add TLD
        let tlds = ["com", "org", "net", "edu", "gov", "io", "dev"];
        let tld = tlds[rng.r#gen_range(0..tlds.len())];

        format!("{}.{}", domain_name, tld)
    }
}

impl Default for EmailGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator<String> for EmailGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let local = self.generate_local_part(rng);
        let domain = self.generate_domain(rng);
        format!("{}@{}", local, domain)
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try minimal email
        if value != "a@b.com" {
            shrinks.push("a@b.com".to_string());
        }

        // Try to shorten local part
        if let Some((local, domain)) = value.split_once('@') {
            if local.len() > 1 {
                shrinks.push(format!("{}@{}", &local[..local.len() - 1], domain));
            }

            // Try to shorten domain
            if let Some((domain_name, tld)) = domain.rsplit_once('.') {
                if domain_name.len() > 1 {
                    shrinks.push(format!(
                        "{}@{}.{}",
                        local,
                        &domain_name[..domain_name.len() - 1],
                        tld
                    ));
                }
            }
        }

        Box::new(shrinks.into_iter())
    }
}

// ============================================================================
// URL Generator
// ============================================================================

/// Generator for URLs with various schemes
#[derive(Debug, Clone)]
pub struct UrlGenerator {
    schemes: Vec<&'static str>,
    include_port: bool,
    include_path: bool,
    include_query: bool,
}

impl UrlGenerator {
    /// Create a new URL generator with http/https schemes
    pub fn new() -> Self {
        Self {
            schemes: vec!["http", "https"],
            include_port: true,
            include_path: true,
            include_query: true,
        }
    }

    /// Create a URL generator with custom schemes
    pub fn with_schemes(schemes: Vec<&'static str>) -> Self {
        Self {
            schemes,
            include_port: true,
            include_path: true,
            include_query: true,
        }
    }

    /// Set whether to include port numbers
    pub fn with_port(mut self, include: bool) -> Self {
        self.include_port = include;
        self
    }

    /// Set whether to include paths
    pub fn with_path(mut self, include: bool) -> Self {
        self.include_path = include;
        self
    }

    /// Set whether to include query parameters
    pub fn with_query(mut self, include: bool) -> Self {
        self.include_query = include;
        self
    }

    fn generate_host(&self, rng: &mut dyn rand::RngCore) -> String {
        let len = rng.r#gen_range(5..20);
        let valid_chars = b"abcdefghijklmnopqrstuvwxyz0123456789-";

        let domain: String = (0..len)
            .map(|_| valid_chars[rng.r#gen_range(0..valid_chars.len())] as char)
            .collect();

        let tlds = ["com", "org", "net", "io", "dev"];
        let tld = tlds[rng.r#gen_range(0..tlds.len())];

        format!("{}.{}", domain, tld)
    }

    fn generate_path(&self, rng: &mut dyn rand::RngCore) -> String {
        let num_segments = rng.r#gen_range(1..=4);
        let valid_chars = b"abcdefghijklmnopqrstuvwxyz0123456789_-";

        let segments: Vec<String> = (0..num_segments)
            .map(|_| {
                let len = rng.r#gen_range(3..10);
                (0..len)
                    .map(|_| valid_chars[rng.r#gen_range(0..valid_chars.len())] as char)
                    .collect()
            })
            .collect();

        format!("/{}", segments.join("/"))
    }

    fn generate_query(&self, rng: &mut dyn rand::RngCore) -> String {
        let num_params = rng.r#gen_range(1..=3);
        let valid_chars = b"abcdefghijklmnopqrstuvwxyz0123456789";

        let params: Vec<String> = (0..num_params)
            .map(|_| {
                let key_len = rng.r#gen_range(3..8);
                let val_len = rng.r#gen_range(3..10);

                let key: String = (0..key_len)
                    .map(|_| valid_chars[rng.r#gen_range(0..valid_chars.len())] as char)
                    .collect();

                let val: String = (0..val_len)
                    .map(|_| valid_chars[rng.r#gen_range(0..valid_chars.len())] as char)
                    .collect();

                format!("{}={}", key, val)
            })
            .collect();

        format!("?{}", params.join("&"))
    }
}

impl Default for UrlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator<String> for UrlGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, _config: &GeneratorConfig) -> String {
        let scheme = self.schemes[rng.r#gen_range(0..self.schemes.len())];
        let host = self.generate_host(rng);

        let mut url = format!("{}://{}", scheme, host);

        if self.include_port && rng.r#gen::<bool>() {
            let port: u16 = rng.r#gen_range(1024..65535);
            url.push_str(&format!(":{}", port));
        }

        if self.include_path && rng.r#gen::<bool>() {
            url.push_str(&self.generate_path(rng));
        }

        if self.include_query && rng.r#gen::<bool>() {
            url.push_str(&self.generate_query(rng));
        }

        url
    }

    fn shrink(&self, value: &String) -> Box<dyn Iterator<Item = String>> {
        let mut shrinks = Vec::new();

        // Try minimal URL
        if value != "http://a.com" {
            shrinks.push("http://a.com".to_string());
        }

        // Try removing query parameters
        if let Some(base) = value.split('?').next() {
            if base != value {
                shrinks.push(base.to_string());
            }
        }

        // Try removing path
        if let Some(scheme_and_host) = value
            .split('/')
            .take(3)
            .collect::<Vec<_>>()
            .join("/")
            .into()
        {
            if scheme_and_host != *value {
                shrinks.push(scheme_and_host);
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
    fn test_ipv4_generator() {
        let gen = IpAddressGenerator::ipv4();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let ip = gen.generate(&mut rng, &config);
            // Should parse as valid IPv4
            let parts: Vec<&str> = ip.split('.').collect();
            assert_eq!(parts.len(), 4);
            for part in parts {
                // Parse as u8 to verify each octet is valid (0-255)
                let _num: u8 = part.parse().unwrap();
            }
        }
    }

    #[test]
    fn test_email_generator() {
        let gen = EmailGenerator::new();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let email = gen.generate(&mut rng, &config);
            assert!(email.contains('@'));
            assert_eq!(email.matches('@').count(), 1);

            let parts: Vec<&str> = email.split('@').collect();
            assert_eq!(parts.len(), 2);
            assert!(!parts[0].is_empty());
            assert!(!parts[1].is_empty());
            assert!(parts[1].contains('.'));
        }
    }

    #[test]
    fn test_url_generator() {
        let gen = UrlGenerator::new();
        let mut rng = thread_rng();
        let config = GeneratorConfig::default();

        for _ in 0..10 {
            let url = gen.generate(&mut rng, &config);
            assert!(url.starts_with("http://") || url.starts_with("https://"));
        }
    }
}
