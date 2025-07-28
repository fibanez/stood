//! Verification framework for LLM Client testing
//!
//! This module re-exports the verification framework from the tests directory
//! when the verification feature is enabled.

#[path = "../tests/provider_integration/mod.rs"]
mod verification_impl;

pub use verification_impl::*;