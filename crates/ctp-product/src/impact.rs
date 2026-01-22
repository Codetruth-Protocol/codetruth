//! Impact Analysis
//!
//! Analyzes the impact of code changes based on product metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::metadata::{ProductMetadata, ComplianceStandard};
use crate::criticality::{CriticalityLevel, CriticalityMap, ReviewLevel};

/// Analyzes impact of code changes
pub struct ImpactAnalyzer {
    metadata: ProductMetadata,
    criticality_map: CriticalityMap,
}

impl ImpactAnalyzer {
    pub fn new(metadata: ProductMetadata, criticality_map: CriticalityMap) -> Self {
        Self { metadata, criticality_map }
    }

    /// Analyze the impact of changed files
    pub fn analyze(&self, changed_files: &[String]) -> ChangeImpactReport {
        let mut affected_components = vec![];
        let mut max_criticality = CriticalityLevel::Low;
        let mut compliance_flags = vec![];

        for file in changed_files {
            let criticality = self.criticality_map.get_criticality(file);
            if criticality > max_criticality {
                max_criticality = criticality;
            }

            affected_components.push(AffectedComponent {
                path: file.clone(),
                criticality,
                is_critical_path: self.metadata.is_critical_path(file),
            });

            // Check compliance scope
            for req in &self.metadata.compliance {
                for scope in &req.scope {
                    if let Ok(pattern) = glob::Pattern::new(scope) {
                        if pattern.matches(file) {
                            compliance_flags.push(ComplianceFlag {
                                standard: req.standard.clone(),
                                file: file.clone(),
                                requires_review: req.standard.requires_security_review(),
                            });
                        }
                    }
                }
            }
        }

        // Calculate user impact
        let user_impact = self.calculate_user_impact(&affected_components);

        // Calculate revenue risk
        let revenue_risk = self.calculate_revenue_risk(&affected_components, max_criticality);

        // Determine recommended review level
        let recommended_review = self.determine_review_level(
            max_criticality,
            &compliance_flags,
            &user_impact,
        );

        // Generate deployment advice
        let deployment_advice = self.generate_deployment_advice(
            max_criticality,
            &user_impact,
            &revenue_risk,
        );

        ChangeImpactReport {
            affected_components,
            max_criticality,
            user_impact,
            revenue_risk,
            compliance_flags,
            recommended_review,
            deployment_advice,
        }
    }

    fn calculate_user_impact(&self, components: &[AffectedComponent]) -> UserImpact {
        let critical_path_affected = components.iter().any(|c| c.is_critical_path);
        let max_criticality = components
            .iter()
            .map(|c| c.criticality)
            .max()
            .unwrap_or(CriticalityLevel::Low);

        // Estimate affected percentage based on criticality
        let affected_percentage = match max_criticality {
            CriticalityLevel::Critical => 1.0,    // Could affect everyone
            CriticalityLevel::High => 0.5,        // Could affect half
            CriticalityLevel::Medium => 0.2,      // Could affect some
            CriticalityLevel::Low => 0.05,        // Minimal impact
        };

        // Handle edge case: zero DAU
        let affected_users = if self.metadata.users.daily_active_users > 0 {
            self.metadata.users.affected_users(affected_percentage)
        } else {
            0
        };

        UserImpact {
            affected_users_estimate: affected_users,
            affected_percentage,
            critical_path_affected,
            daily_active_users: self.metadata.users.daily_active_users,
        }
    }

    fn calculate_revenue_risk(
        &self,
        components: &[AffectedComponent],
        max_criticality: CriticalityLevel,
    ) -> RevenueRisk {
        let revenue = match &self.metadata.revenue {
            Some(r) => r,
            None => return RevenueRisk::default(),
        };

        let revenue_path_affected = components.iter().any(|c| {
            revenue.critical_paths.iter().any(|rp| {
                glob::Pattern::new(rp)
                    .map(|p| p.matches(&c.path))
                    .unwrap_or(false)
            })
        });

        let risk_level = if revenue_path_affected {
            RiskLevel::High
        } else {
            match max_criticality {
                CriticalityLevel::Critical => RiskLevel::High,
                CriticalityLevel::High => RiskLevel::Medium,
                CriticalityLevel::Medium => RiskLevel::Low,
                CriticalityLevel::Low => RiskLevel::Minimal,
            }
        };

        let downtime_cost = self.metadata.downtime_cost_per_minute().unwrap_or(0.0);

        RevenueRisk {
            risk_level,
            monthly_value_at_risk: revenue.monthly_value,
            downtime_cost_per_minute: downtime_cost,
            revenue_path_affected,
            currency: revenue.currency.clone(),
        }
    }

    fn determine_review_level(
        &self,
        max_criticality: CriticalityLevel,
        compliance_flags: &[ComplianceFlag],
        user_impact: &UserImpact,
    ) -> ReviewLevel {
        // Check for compliance requirements
        if compliance_flags.iter().any(|f| f.requires_review) {
            return ReviewLevel::Regulated;
        }

        // Check criticality
        if max_criticality == CriticalityLevel::Critical {
            return ReviewLevel::Critical;
        }

        // Check user impact
        if user_impact.critical_path_affected {
            return ReviewLevel::Critical;
        }

        if user_impact.affected_percentage > 0.3 {
            return ReviewLevel::Enhanced;
        }

        max_criticality.review_level()
    }

    fn generate_deployment_advice(
        &self,
        max_criticality: CriticalityLevel,
        user_impact: &UserImpact,
        revenue_risk: &RevenueRisk,
    ) -> DeploymentAdvice {
        let deployment = &self.metadata.deployment;

        // Determine strategy
        let (strategy, canary_percentage, monitoring_duration_hours) = 
            if max_criticality == CriticalityLevel::Critical || revenue_risk.risk_level == RiskLevel::High {
                if deployment.supports_gradual_rollout {
                    (DeploymentStrategy::Canary, Some(1), 4)
                } else {
                    (DeploymentStrategy::ManualVerification, None, 2)
                }
            } else if max_criticality == CriticalityLevel::High {
                if deployment.supports_gradual_rollout {
                    (DeploymentStrategy::Canary, Some(5), 2)
                } else {
                    (DeploymentStrategy::Rolling, None, 1)
                }
            } else {
                (DeploymentStrategy::Standard, None, 0)
            };

        // Generate rollback triggers
        let mut rollback_triggers = vec![
            "Error rate > 0.1%".to_string(),
            format!("P99 latency > {}ms", if max_criticality >= CriticalityLevel::High { 500 } else { 1000 }),
        ];

        if revenue_risk.revenue_path_affected {
            rollback_triggers.push("Payment failure rate > 0.5%".to_string());
        }

        if user_impact.critical_path_affected {
            rollback_triggers.push("Critical path availability < 99.9%".to_string());
        }

        DeploymentAdvice {
            strategy,
            canary_percentage,
            canary_stages: if canary_percentage.is_some() {
                vec![1, 5, 25, 100]
            } else {
                vec![]
            },
            monitoring_duration_hours,
            rollback_triggers,
            estimated_rollback_minutes: deployment.rollback_time_minutes,
            requires_manual_approval: max_criticality >= CriticalityLevel::Critical,
        }
    }
}

/// Report of change impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeImpactReport {
    /// Components affected by the change
    pub affected_components: Vec<AffectedComponent>,
    
    /// Maximum criticality level among affected components
    pub max_criticality: CriticalityLevel,
    
    /// User impact assessment
    pub user_impact: UserImpact,
    
    /// Revenue risk assessment
    pub revenue_risk: RevenueRisk,
    
    /// Compliance flags triggered
    pub compliance_flags: Vec<ComplianceFlag>,
    
    /// Recommended review level
    pub recommended_review: ReviewLevel,
    
    /// Deployment recommendations
    pub deployment_advice: DeploymentAdvice,
}

impl ChangeImpactReport {
    /// Generate a summary string
    pub fn summary(&self) -> String {
        let mut lines = vec![];
        
        lines.push(format!(
            "Impact: {:?} | Review: {:?}",
            self.max_criticality, self.recommended_review
        ));
        
        lines.push(format!(
            "Users affected: ~{} ({:.0}%)",
            self.user_impact.affected_users_estimate,
            self.user_impact.affected_percentage * 100.0
        ));
        
        if self.revenue_risk.revenue_path_affected {
            lines.push(format!(
                "Revenue risk: {:?} (${:.0}/min downtime)",
                self.revenue_risk.risk_level,
                self.revenue_risk.downtime_cost_per_minute
            ));
        }
        
        if !self.compliance_flags.is_empty() {
            let standards: Vec<_> = self.compliance_flags
                .iter()
                .map(|f| format!("{:?}", f.standard))
                .collect();
            lines.push(format!("Compliance: {}", standards.join(", ")));
        }
        
        lines.push(format!(
            "Deploy: {:?}{}",
            self.deployment_advice.strategy,
            if self.deployment_advice.requires_manual_approval {
                " (manual approval required)"
            } else {
                ""
            }
        ));
        
        lines.join("\n")
    }
}

/// An affected component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedComponent {
    pub path: String,
    pub criticality: CriticalityLevel,
    pub is_critical_path: bool,
}

/// User impact assessment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserImpact {
    /// Estimated number of affected users
    pub affected_users_estimate: u64,
    
    /// Percentage of DAU affected
    pub affected_percentage: f64,
    
    /// Whether a critical user path is affected
    pub critical_path_affected: bool,
    
    /// Total daily active users for context
    pub daily_active_users: u64,
}

/// Revenue risk assessment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevenueRisk {
    /// Risk level
    pub risk_level: RiskLevel,
    
    /// Monthly value at risk
    pub monthly_value_at_risk: f64,
    
    /// Cost per minute of downtime
    pub downtime_cost_per_minute: f64,
    
    /// Whether revenue-critical path is affected
    pub revenue_path_affected: bool,
    
    /// Currency
    pub currency: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Minimal,
    Low,
    Medium,
    High,
}

/// Compliance flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFlag {
    pub standard: ComplianceStandard,
    pub file: String,
    pub requires_review: bool,
}

/// Deployment advice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentAdvice {
    /// Recommended deployment strategy
    pub strategy: DeploymentStrategy,
    
    /// Initial canary percentage (if applicable)
    pub canary_percentage: Option<u8>,
    
    /// Canary stages (percentages)
    pub canary_stages: Vec<u8>,
    
    /// Hours to monitor at each stage
    pub monitoring_duration_hours: u32,
    
    /// Conditions that should trigger rollback
    pub rollback_triggers: Vec<String>,
    
    /// Estimated rollback time in minutes
    pub estimated_rollback_minutes: u32,
    
    /// Whether manual approval is required
    pub requires_manual_approval: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStrategy {
    /// Standard deployment
    Standard,
    /// Rolling deployment
    Rolling,
    /// Canary deployment
    Canary,
    /// Blue-green deployment
    BlueGreen,
    /// Requires manual verification before proceeding
    ManualVerification,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{UserMetrics, RevenueInfo, RevenueModel, DeploymentInfo};

    fn create_test_metadata() -> ProductMetadata {
        ProductMetadata {
            name: "Test App".to_string(),
            users: UserMetrics {
                total_users: 1_000_000,
                daily_active_users: 100_000,
                monthly_active_users: 500_000,
                peak_concurrent_users: 10_000,
                segments: vec![],
            },
            revenue: Some(RevenueInfo {
                model: RevenueModel::Subscription,
                monthly_value: 1_000_000.0,
                currency: "USD".to_string(),
                critical_paths: vec!["**/payments/**".to_string()],
            }),
            deployment: DeploymentInfo {
                supports_gradual_rollout: true,
                rollback_time_minutes: 5,
                ..Default::default()
            },
            critical_paths: vec!["**/checkout/**".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn test_impact_analysis_critical() {
        let metadata = create_test_metadata();
        let criticality_map = CriticalityMap::with_defaults();
        let analyzer = ImpactAnalyzer::new(metadata, criticality_map);

        let changed_files = vec!["src/payments/stripe.ts".to_string()];
        let report = analyzer.analyze(&changed_files);

        assert_eq!(report.max_criticality, CriticalityLevel::Critical);
        assert!(report.revenue_risk.revenue_path_affected);
        assert!(report.deployment_advice.requires_manual_approval);
    }

    #[test]
    fn test_impact_analysis_low() {
        let metadata = create_test_metadata();
        let criticality_map = CriticalityMap::with_defaults();
        let analyzer = ImpactAnalyzer::new(metadata, criticality_map);

        let changed_files = vec!["src/marketing/banner.ts".to_string()];
        let report = analyzer.analyze(&changed_files);

        assert_eq!(report.max_criticality, CriticalityLevel::Low);
        assert!(!report.deployment_advice.requires_manual_approval);
    }

    #[test]
    fn test_deployment_advice_canary() {
        let metadata = create_test_metadata();
        let criticality_map = CriticalityMap::with_defaults();
        let analyzer = ImpactAnalyzer::new(metadata, criticality_map);

        let changed_files = vec!["src/payments/checkout.ts".to_string()];
        let report = analyzer.analyze(&changed_files);

        assert!(matches!(report.deployment_advice.strategy, DeploymentStrategy::Canary));
        assert!(report.deployment_advice.canary_percentage.is_some());
    }
}
