//! # F1 Strategy Engine
//!
//! A high-performance, real-time Formula 1 pit strategy simulation and telemetry optimization platform.
//!
//! This crate provides domain models, physics degradation calculators, FIA sporting regulation
//! validators, strategy permutation optimizers, and Axum HTTP web server endpoints.

pub mod api;
pub mod config;
pub mod domain;
pub mod engine;
pub mod error;

