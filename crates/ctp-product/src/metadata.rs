//! Product Metadata
//!
//! Core data structures for product sensitivity information.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete product metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMetadata {
    /// Product name
    pub name: String,
    
    /// Product type (b2c_saas, b2b_enterprise, internal_tool, etc.)
    pub product_type: ProductType,
    
    /// User metrics
    pub users: UserMetrics,
    
    /// Revenue information
    pub revenue: Option<RevenueInfo>,
    
    /// Deployment architecture
    pub deployment: DeploymentInfo,
    
    /// Compliance requirements
    pub compliance: Vec<ComplianceRequirement>,
    
    /// Critical paths in the application
    pub critical_paths: Vec<String>,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ProductMetadata {
    /// Load from a YAML file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let metadata: Self = serde_yaml::from_str(&content)?;
        Ok(metadata)
    }

    /// Save to a YAML file
    pub fn to_file(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check if a path is in a critical path
    pub fn is_critical_path(&self, file_path: &str) -> bool {
        self.critical_paths.iter().any(|cp| {
            let pattern = glob::Pattern::new(cp).ok();
            pattern.map(|p| p.matches(file_path)).unwrap_or(false)
        })
    }

    /// Get estimated cost per minute of downtime
    pub fn downtime_cost_per_minute(&self) -> Option<f64> {
        let revenue = self.revenue.as_ref()?;
        
        // More accurate calculation: use average month length (30.44 days)
        let minutes_per_month = 30.44 * 24.0 * 60.0;
        Some(revenue.monthly_value / minutes_per_month)
    }
}

impl Default for ProductMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown Product".to_string(),
            product_type: ProductType::Unknown,
            users: UserMetrics::default(),
            revenue: None,
            deployment: DeploymentInfo::default(),
            compliance: vec![],
            critical_paths: vec![],
            metadata: serde_json::json!({}),
        }
    }
}

/// Product type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    /// Business to consumer SaaS
    B2cSaas,
    /// Business to business enterprise
    B2bEnterprise,
    /// Internal tool
    InternalTool,
    /// Mobile application
    MobileApp,
    /// API/Platform
    ApiPlatform,
    /// E-commerce
    Ecommerce,
    /// Financial services
    FinancialServices,
    /// Healthcare
    Healthcare,
    /// Unknown
    Unknown,
}

impl ProductType {
    /// Get base risk multiplier for this product type
    pub fn risk_multiplier(&self) -> f64 {
        match self {
            ProductType::FinancialServices => 2.0,
            ProductType::Healthcare => 2.0,
            ProductType::Ecommerce => 1.5,
            ProductType::B2cSaas => 1.2,
            ProductType::B2bEnterprise => 1.3,
            ProductType::MobileApp => 1.4, // Harder to roll back
            ProductType::ApiPlatform => 1.5, // Many dependents
            ProductType::InternalTool => 0.8,
            ProductType::Unknown => 1.0,
        }
    }
}

/// User metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMetrics {
    /// Total registered users
    pub total_users: u64,
    
    /// Daily active users
    pub daily_active_users: u64,
    
    /// Monthly active users
    pub monthly_active_users: u64,
    
    /// Peak concurrent users
    pub peak_concurrent_users: u64,
    
    /// User segments (optional breakdown)
    #[serde(default)]
    pub segments: Vec<UserSegment>,
}

impl UserMetrics {
    /// Estimate affected users for a given percentage
    pub fn affected_users(&self, percentage: f64) -> u64 {
        (self.daily_active_users as f64 * percentage) as u64
    }
}

/// User segment for more granular impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSegment {
    pub name: String,
    pub count: u64,
    pub revenue_contribution: Option<f64>,
    pub priority: SegmentPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SegmentPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Revenue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueInfo {
    /// Revenue model
    pub model: RevenueModel,
    
    /// Monthly revenue/GMV value
    pub monthly_value: f64,
    
    /// Currency
    #[serde(default = "default_currency")]
    pub currency: String,
    
    /// Revenue-critical paths
    #[serde(default)]
    pub critical_paths: Vec<String>,
}

fn default_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevenueModel {
    Subscription,
    TransactionFee,
    Advertising,
    Licensing,
    Freemium,
    Usage,
    Hybrid,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Deployment type
    pub deployment_type: DeploymentType,
    
    /// Regions deployed to
    #[serde(default)]
    pub regions: Vec<String>,
    
    /// Rollout strategy
    pub rollout_strategy: RolloutStrategy,
    
    /// Average rollback time in minutes
    pub rollback_time_minutes: u32,
    
    /// Whether blue-green or canary is available
    pub supports_gradual_rollout: bool,
}

impl Default for DeploymentInfo {
    fn default() -> Self {
        Self {
            deployment_type: DeploymentType::Unknown,
            regions: vec![],
            rollout_strategy: RolloutStrategy::AllAtOnce,
            rollback_time_minutes: 30,
            supports_gradual_rollout: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentType {
    Kubernetes,
    Serverless,
    VirtualMachines,
    Containers,
    BareMetal,
    Hybrid,
    MobileAppStore,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStrategy {
    AllAtOnce,
    Canary,
    BlueGreen,
    Rolling,
    FeatureFlags,
}

impl RolloutStrategy {
    /// Get risk reduction factor for this strategy
    pub fn risk_reduction(&self) -> f64 {
        match self {
            RolloutStrategy::AllAtOnce => 1.0,
            RolloutStrategy::Rolling => 0.8,
            RolloutStrategy::Canary => 0.5,
            RolloutStrategy::BlueGreen => 0.4,
            RolloutStrategy::FeatureFlags => 0.3,
        }
    }
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    /// Compliance standard
    pub standard: ComplianceStandard,
    
    /// Scope (paths that are in scope)
    #[serde(default)]
    pub scope: Vec<String>,
    
    /// Additional requirements
    #[serde(default)]
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceStandard {
    PciDss,
    Gdpr,
    Ccpa,
    Hipaa,
    Sox,
    Iso27001,
    Soc2,
    Fedramp,
    Other(String),
}

impl ComplianceStandard {
    /// Get review requirements for this standard
    pub fn requires_security_review(&self) -> bool {
        matches!(
            self,
            ComplianceStandard::PciDss
                | ComplianceStandard::Hipaa
                | ComplianceStandard::Sox
                | ComplianceStandard::Fedramp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_type_risk() {
        assert!(ProductType::FinancialServices.risk_multiplier() > ProductType::InternalTool.risk_multiplier());
    }

    #[test]
    fn test_downtime_cost() {
        let metadata = ProductMetadata {
            revenue: Some(RevenueInfo {
                model: RevenueModel::Subscription,
                monthly_value: 1_000_000.0,
                currency: "USD".to_string(),
                critical_paths: vec![],
            }),
            ..Default::default()
        };

        let cost = metadata.downtime_cost_per_minute().unwrap();
        assert!(cost > 0.0);
        // ~$23/minute for $1M/month
        assert!(cost > 20.0 && cost < 30.0);
    }

    #[test]
    fn test_critical_path_matching() {
        let metadata = ProductMetadata {
            critical_paths: vec![
                "src/payments/**".to_string(),
                "src/auth/**".to_string(),
            ],
            ..Default::default()
        };

        assert!(metadata.is_critical_path("src/payments/stripe.ts"));
        assert!(metadata.is_critical_path("src/auth/login.ts"));
        assert!(!metadata.is_critical_path("src/marketing/banner.ts"));
    }
}
