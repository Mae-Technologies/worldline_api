//! Serde models for the [Worldline NAM Payment API](https://docs.na.worldline-solutions.com/).
//!
//! `worldline_api` is a standalone Rust library with no Mae or Statbook dependencies.
//! Use it to build request bodies, parse responses, and validate payment-profile wire
//! data for any Worldline merchant integration.
//!
//! # Modules
//!
//! - [`profile`] — payment profiles (billing, bank account, cards)
//! - [`batch`] — batch settlement criteria and responses
//!
//! # Example
//!
//! ```rust
//! use worldline_api::profile::{format_card_label, redact_card_number};
//!
//! assert_eq!(redact_card_number("4030001234567890"), "4030***7890");
//! assert_eq!(
//!     format_card_label(Some("VI"), "4030001234567890", ""),
//!     "Visa 4030***7890 card on file"
//! );
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod batch;
pub mod profile;
