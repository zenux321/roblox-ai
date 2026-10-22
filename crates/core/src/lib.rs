//! Nucleul Roblox AI Builder: contracte, validare strictă, policy engine,
//! mașina de stări, furnizor fals și executor demonstrativ.
//!
//! Testabil fără Windows. Nu conține rețea, secrete sau shell.

pub mod approval;
pub mod config;
pub mod examples;
pub mod executor;
pub mod ids;
pub mod paths;
pub mod plan;
pub mod policy;
pub mod provider;
pub mod state;

/// Versiunea nucleului, afișată în UI.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
