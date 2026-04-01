use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use axum::{
    http::StatusCode,
    response::Json,
    extract::State,
};
use serde_json::json;

/// Status code statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCodeStats {
    pub code: u16,
    pub count: u64,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub category: String,
}

/// Request statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStats {
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub average_response_time: Duration,
    pub last_request: chrono::DateTime<chrono::Utc>,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub database_connected: bool,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_connections: u32,
}

/// Status monitor for tracking API health and statistics
#[derive(Debug)]
pub struct StatusMonitor {
    start_time: Instant,
    status_codes: Arc<Mutex<HashMap<u16, StatusCodeStats>>>,
    request_times: Arc<Mutex<Vec<Duration>>>,
    total_requests: Arc<Mutex<u64>>,
    last_request_time: Arc<Mutex<Option<Instant>>>,
    health_status: Arc<Mutex<HealthStatus>>,
}

impl StatusMonitor {
    pub fn new() -> Self {
        let monitor = Self {
            start_time: Instant::now(),
            status_codes: Arc::new(Mutex::new(HashMap::new())),
            request_times: Arc::new(Mutex::new(Vec::new())),
            total_requests: Arc::new(Mutex::new(0)),
            last_request_time: Arc::new(Mutex::new(None)),
            health_status: Arc::new(Mutex::new(HealthStatus {
                status: "healthy".to_string(),
                database_connected: false,
                uptime_seconds: 0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                active_connections: 0,
            })),
        };

        // Start background monitoring task
        monitor.start_monitoring();
        monitor
    }

    /// Record a status code occurrence
    pub fn record_status_code(&self, code: StatusCode) {
        let mut status_codes = self.status_codes.lock().unwrap();
        let entry = status_codes.entry(code.as_u16()).or_insert_with(|| StatusCodeStats {
            code: code.as_u16(),
            count: 0,
            last_seen: chrono::Utc::now(),
            category: self.get_status_category(code),
        });
        
        entry.count += 1;
        entry.last_seen = chrono::Utc::now();

        // Update total requests
        *self.total_requests.lock().unwrap() += 1;
        *self.last_request_time.lock().unwrap() = Some(Instant::now());

        // Log based on status code category
        match code.as_u16() {
            200..=299 => debug!(status_code = code.as_u16(), "Success response"),
            300..=399 => debug!(status_code = code.as_u16(), "Redirect response"),
            400..=499 => warn!(status_code = code.as_u16(), "Client error"),
            500..=599 => error!(status_code = code.as_u16(), "Server error"),
            _ => info!(status_code = code.as_u16(), "Other response"),
        }
    }

    /// Record request response time
    pub fn record_request_time(&self, duration: Duration) {
        let mut request_times = self.request_times.lock().unwrap();
        request_times.push(duration);
        
        // Keep only last 1000 requests for average calculation
        if request_times.len() > 1000 {
            request_times.remove(0);
        }
    }

    /// Get status code statistics
    pub fn get_status_code_stats(&self) -> Vec<StatusCodeStats> {
        let status_codes = self.status_codes.lock().unwrap();
        status_codes.values().cloned().collect()
    }

    /// Get request statistics
    pub fn get_request_stats(&self) -> RequestStats {
        let total_requests = *self.total_requests.lock().unwrap();
        let request_times = self.request_times.lock().unwrap();
        let last_request = *self.last_request_time.lock().unwrap();
        
        let average_response_time = if request_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = request_times.iter().sum();
            total / request_times.len() as u32
        };

        let uptime = self.start_time.elapsed();
        let requests_per_second = if uptime.as_secs() > 0 {
            total_requests as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        RequestStats {
            total_requests,
            requests_per_second,
            average_response_time,
            last_request: last_request
                .map(|_| chrono::Utc::now())
                .unwrap_or_else(|| chrono::Utc::now()),
        }
    }

    /// Get current health status
    pub fn get_health_status(&self) -> HealthStatus {
        let health_status = self.health_status.lock().unwrap();
        health_status.clone()
    }

    /// Update health status
    pub fn update_health_status(&self, status: HealthStatus) {
        *self.health_status.lock().unwrap() = status;
    }

    /// Get status category based on HTTP status code
    fn get_status_category(&self, code: StatusCode) -> String {
        match code.as_u16() {
            200..=299 => "success".to_string(),
            300..=399 => "redirect".to_string(),
            400..=499 => "client_error".to_string(),
            500..=599 => "server_error".to_string(),
            _ => "other".to_string(),
        }
    }

    /// Start background monitoring task
    fn start_monitoring(&self) {
        let status_codes = self.status_codes.clone();
        let request_times = self.request_times.clone();
        let total_requests = self.total_requests.clone();
        let health_status = self.health_status.clone();
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Update every 30 seconds
            
            loop {
                interval.tick().await;
                
                let uptime = start_time.elapsed();
                let current_total = *total_requests.lock().unwrap();
                let current_request_times = request_times.lock().unwrap().len();
                
                // Update health status
                let mut health = health_status.lock().unwrap();
                health.uptime_seconds = uptime.as_secs();
                
                // Update system metrics (simplified - in production, use proper system monitoring)
                health.memory_usage_mb = self.get_memory_usage();
                health.cpu_usage_percent = self.get_cpu_usage();
                health.active_connections = current_request_times as u32;
                
                // Determine overall health
                let error_rate = self.calculate_error_rate();
                health.status = if error_rate > 0.1 {
                    "degraded".to_string()
                } else if error_rate > 0.5 {
                    "unhealthy".to_string()
                } else {
                    "healthy".to_string()
                };
                
                info!(
                    uptime_seconds = uptime.as_secs(),
                    total_requests = current_total,
                    error_rate = error_rate,
                    memory_usage_mb = health.memory_usage_mb,
                    cpu_usage_percent = health.cpu_usage_percent,
                    "Status monitor update"
                );
            }
        });
    }

    /// Calculate error rate (4xx and 5xx responses)
    fn calculate_error_rate(&self) -> f64 {
        let status_codes = self.status_codes.lock().unwrap();
        let total_requests = *self.total_requests.lock().unwrap();
        
        if total_requests == 0 {
            return 0.0;
        }

        let error_count: u64 = status_codes
            .values()
            .filter(|stats| stats.code >= 400)
            .map(|stats| stats.count)
            .sum();

        error_count as f64 / total_requests as f64
    }

    /// Get memory usage (simplified)
    fn get_memory_usage(&self) -> f64 {
        // In production, use proper system monitoring libraries
        // For now, return a placeholder value
        50.0 // MB
    }

    /// Get CPU usage (simplified)
    fn get_cpu_usage(&self) -> f64 {
        // In production, use proper system monitoring libraries
        // For now, return a placeholder value
        25.0 // Percentage
    }

    /// Get comprehensive status report
    pub fn get_status_report(&self) -> serde_json::Value {
        json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_seconds": self.start_time.elapsed().as_secs(),
            "health": self.get_health_status(),
            "requests": self.get_request_stats(),
            "status_codes": self.get_status_code_stats(),
            "top_errors": self.get_top_errors(),
            "performance": {
                "average_response_time_ms": self.get_request_stats().average_response_time.as_millis(),
                "requests_per_second": self.get_request_stats().requests_per_second,
                "error_rate": self.calculate_error_rate()
            }
        })
    }

    /// Get top error codes
    fn get_top_errors(&self) -> Vec<StatusCodeStats> {
        let mut errors: Vec<_> = self
            .get_status_code_stats()
            .into_iter()
            .filter(|stats| stats.code >= 400)
            .collect();
        
        errors.sort_by(|a, b| b.count.cmp(&a.count));
        errors.truncate(10); // Top 10 errors
        errors
    }
}

/// Middleware to track status codes and request times
pub async fn status_monitoring_middleware(
    State(monitor): State<Arc<StatusMonitor>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let start_time = Instant::now();
    let response = next.run(request).await;
    let duration = start_time.elapsed();
    
    monitor.record_status_code(response.status());
    monitor.record_request_time(duration);
    
    Ok(response)
}

/// API endpoint to get status report
pub async fn get_status_report(
    State(monitor): State<Arc<StatusMonitor>>,
) -> Json<serde_json::Value> {
    Json(monitor.get_status_report())
}

/// API endpoint to get health status
pub async fn get_health_check(
    State(monitor): State<Arc<StatusMonitor>>,
) -> Json<serde_json::Value> {
    let health = monitor.get_health_status();
    let status_code = if health.status == "healthy" { 200 } else { 503 };
    
    Json(json!({
        "status": health.status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": health.uptime_seconds,
        "database_connected": health.database_connected,
        "memory_usage_mb": health.memory_usage_mb,
        "cpu_usage_percent": health.cpu_usage_percent,
        "active_connections": health.active_connections
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_status_monitor() {
        let monitor = StatusMonitor::new();
        
        // Record some status codes
        monitor.record_status_code(StatusCode::OK);
        monitor.record_status_code(StatusCode::NOT_FOUND);
        monitor.record_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        
        let stats = monitor.get_status_code_stats();
        assert_eq!(stats.len(), 3);
        
        let health = monitor.get_health_status();
        assert_eq!(health.status, "healthy");
    }

    #[test]
    fn test_status_category() {
        let monitor = StatusMonitor::new();
        
        assert_eq!(monitor.get_status_category(StatusCode::OK), "success");
        assert_eq!(monitor.get_status_category(StatusCode::NOT_FOUND), "client_error");
        assert_eq!(monitor.get_status_category(StatusCode::INTERNAL_SERVER_ERROR), "server_error");
    }
}
