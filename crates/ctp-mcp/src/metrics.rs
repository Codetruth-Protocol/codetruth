//! Metrics instrumentation for MCP server
//!
//! Provides structured metrics for observability and monitoring
//! including timing, counts, and error rates.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Metrics collector for MCP operations
#[derive(Debug, Default)]
pub struct MetricsCollector {
    /// Total number of analysis requests
    pub analysis_requests: AtomicU64,
    /// Total number of successful analyses
    pub analysis_success: AtomicU64,
    /// Total number of failed analyses
    pub analysis_failures: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// Total files analyzed
    pub files_analyzed: AtomicU64,
    /// Total stubs detected
    pub stubs_detected: AtomicU64,
    /// Analysis duration histogram buckets (in ms)
    pub duration_under_100ms: AtomicU64,
    pub duration_100ms_to_1s: AtomicU64,
    pub duration_1s_to_5s: AtomicU64,
    pub duration_5s_to_30s: AtomicU64,
    pub duration_over_30s: AtomicU64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an analysis request
    pub fn record_request(&self) {
        self.analysis_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful analysis
    pub fn record_success(&self) {
        self.analysis_success.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed analysis
    pub fn record_failure(&self) {
        self.analysis_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record files analyzed
    pub fn record_files_analyzed(&self, count: u64) {
        self.files_analyzed.fetch_add(count, Ordering::Relaxed);
    }

    /// Record stubs detected
    pub fn record_stubs(&self, count: u64) {
        self.stubs_detected.fetch_add(count, Ordering::Relaxed);
    }

    /// Record analysis duration
    pub fn record_duration(&self, duration_ms: u64) {
        match duration_ms {
            0..=100 => self.duration_under_100ms.fetch_add(1, Ordering::Relaxed),
            101..=1000 => self.duration_100ms_to_1s.fetch_add(1, Ordering::Relaxed),
            1001..=5000 => self.duration_1s_to_5s.fetch_add(1, Ordering::Relaxed),
            5001..=30000 => self.duration_5s_to_30s.fetch_add(1, Ordering::Relaxed),
            _ => self.duration_over_30s.fetch_add(1, Ordering::Relaxed),
        };
    }

    /// Get cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let misses = self.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total == 0.0 {
            0.0
        } else {
            (hits / total) * 100.0
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        let success = self.analysis_success.load(Ordering::Relaxed) as f64;
        let failures = self.analysis_failures.load(Ordering::Relaxed) as f64;
        let total = success + failures;
        if total == 0.0 {
            0.0
        } else {
            (success / total) * 100.0
        }
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            analysis_requests: self.analysis_requests.load(Ordering::Relaxed),
            analysis_success: self.analysis_success.load(Ordering::Relaxed),
            analysis_failures: self.analysis_failures.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            files_analyzed: self.files_analyzed.load(Ordering::Relaxed),
            stubs_detected: self.stubs_detected.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            success_rate: self.success_rate(),
            duration_distribution: DurationDistribution {
                under_100ms: self.duration_under_100ms.load(Ordering::Relaxed),
                ms_100_to_1s: self.duration_100ms_to_1s.load(Ordering::Relaxed),
                s_1_to_5s: self.duration_1s_to_5s.load(Ordering::Relaxed),
                s_5_to_30s: self.duration_5s_to_30s.load(Ordering::Relaxed),
                over_30s: self.duration_over_30s.load(Ordering::Relaxed),
            },
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub analysis_requests: u64,
    pub analysis_success: u64,
    pub analysis_failures: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub files_analyzed: u64,
    pub stubs_detected: u64,
    pub cache_hit_rate: f64,
    pub success_rate: f64,
    pub duration_distribution: DurationDistribution,
}

/// Duration distribution histogram
#[derive(Debug, Clone, Copy)]
pub struct DurationDistribution {
    pub under_100ms: u64,
    pub ms_100_to_1s: u64,
    pub s_1_to_5s: u64,
    pub s_5_to_30s: u64,
    pub over_30s: u64,
}

impl DurationDistribution {
    /// Calculate average duration estimate (in ms)
    pub fn estimated_avg_ms(&self) -> u64 {
        let total = self.under_100ms + self.ms_100_to_1s + self.s_1_to_5s + self.s_5_to_30s + self.over_30s;
        if total == 0 {
            return 0;
        }
        // Use midpoint of each bucket for estimation
        let weighted_sum = self.under_100ms * 50 +
            self.ms_100_to_1s * 550 +
            self.s_1_to_5s * 3000 +
            self.s_5_to_30s * 17500 +
            self.over_30s * 60000;
        weighted_sum / total
    }
}

/// Timing guard that records duration on drop
pub struct TimingGuard<'a> {
    start: Instant,
    metrics: &'a MetricsCollector,
}

impl<'a> TimingGuard<'a> {
    /// Create a new timing guard
    pub fn new(metrics: &'a MetricsCollector) -> Self {
        Self {
            start: Instant::now(),
            metrics,
        }
    }

    /// Get elapsed time
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

impl<'a> Drop for TimingGuard<'a> {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        self.metrics.record_duration(duration_ms);
    }
}

/// Health status of the MCP server
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: &'static str,
    pub uptime_seconds: u64,
    pub engine_ready: bool,
    pub cache_size: usize,
    pub metrics: MetricsSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = MetricsCollector::new();
        
        metrics.record_request();
        metrics.record_request();
        metrics.record_success();
        metrics.record_failure();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.analysis_requests, 2);
        assert_eq!(snapshot.analysis_success, 1);
        assert_eq!(snapshot.analysis_failures, 1);
        assert_eq!(snapshot.cache_hits, 1);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_rate, 50.0);
    }

    #[test]
    fn test_duration_distribution() {
        let dist = DurationDistribution {
            under_100ms: 10,
            ms_100_to_1s: 5,
            s_1_to_5s: 3,
            s_5_to_30s: 2,
            over_30s: 1,
        };
        
        let avg = dist.estimated_avg_ms();
        assert!(avg > 0);
        assert!(avg < 60000); // Should be reasonable
    }

    #[test]
    fn test_timing_guard() {
        let metrics = MetricsCollector::new();
        
        {
            let _guard = TimingGuard::new(&metrics);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        let snapshot = metrics.snapshot();
        // Duration should be recorded in under_100ms bucket
        assert!(snapshot.duration_distribution.under_100ms >= 1);
    }
}
