//! Lucid API service.
//!
//! Provides REST API endpoints for fleet management, authentication, and telemetry.
//!
//! # Configuration
//!
//! The API requires an Ed25519 signing key for session authentication. See
//! [`config::LucidApiConfig`] for configuration options.
//!
//! # Authentication
//!
//! Session-based authentication using Ed25519 signatures. See [`auth::signing`]
//! for implementation details.

pub mod auth;
pub mod config;
pub mod server;

pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod handlers;
