//! # CTP Product Sensitivity
//!
//! Product metadata and impact analysis for CodeTruth Protocol.
//!
//! This crate provides:
//! - Product metadata schema (users, revenue, deployment, compliance)
//! - Component criticality mapping
//! - Change impact analysis
//! - Deployment recommendations based on risk

pub mod metadata;
pub mod criticality;
pub mod impact;

pub use metadata::{ProductMetadata, UserMetrics, RevenueInfo, DeploymentInfo, ComplianceRequirement};
pub use criticality::{CriticalityLevel, CriticalityMap, ComponentCriticality};
pub use impact::{ImpactAnalyzer, ChangeImpactReport, UserImpact, RevenueRisk, DeploymentAdvice};
