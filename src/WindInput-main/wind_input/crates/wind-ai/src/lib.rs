//! Selection translation / polishing for WindInput.
//!
//! The crate deliberately owns only the portable pieces: prompts, the
//! OpenAI-compatible HTTP call, and the encrypted API-key file. Selection
//! capture and UI live in the coordinator/UI layers.

pub mod client;
pub mod prompt;
pub mod secret;

pub use client::{AiError, AiResult, translate, polish};
pub use secret::{clear_api_key, has_api_key, load_api_key, save_api_key};
