# Status Code Logger and Monitoring System

## 🎯 Overview

The Game Server Panel backend now includes a comprehensive status code logging and monitoring system that provides real-time insights into API performance, error rates, and system health.

## 📊 Features

### 1. **Custom Error Handling**
- Structured error types with automatic HTTP status code mapping
- Detailed error logging with appropriate log levels
- Consistent error response format
- Error categorization (database, auth, validation, etc.)

### 2. **Status Code Tracking**
- Real-time monitoring of all HTTP status codes
- Automatic categorization (success, client_error, server_error)
- Error rate calculations
- Top error identification

### 3. **Performance Monitoring**
- Request timing and average response times
- Requests per second tracking
- Memory and CPU usage monitoring
- Uptime tracking

### 4. **Health Checks**
- Comprehensive health status reporting
- Database connectivity monitoring
- System resource monitoring
- Service availability status

## 🚀 Getting Started

### Run the Improved Server

```bash
# Start database
make docker-up
make db-wait

# Run improved server with status monitoring
make run-improved
```

### Available Endpoints

```bash
# API root with information
GET /

# Basic health check
GET /health

# Detailed status report
GET /status

# API endpoints with enhanced error handling
POST /api/v1/auth/register
POST /api/v1/auth/login
GET  /api/v1/servers
POST /api/v1/servers
GET  /api/v1/servers/:id
POST /api/v1/servers/:id/start
POST /api/v1/servers/:id/stop
POST /api/v1/servers/:id/restart
GET  /api/v1/servers/:id/metrics
```

## 📈 Response Formats

### Success Response
```json
{
  "success": true,
  "data": { ... },
  "message": "Operation successful",
  "meta": {
    "version": "1.0",
    "count": 10
  },
  "timestamp": "2026-04-01T15:30:00Z"
}
```

### Error Response
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed: username is required",
    "category": "validation"
  },
  "timestamp": "2026-04-01T15:30:00Z",
  "path": "/api/v1/auth/register"
}
```

### Health Check Response
```json
{
  "status": "healthy",
  "timestamp": "2026-04-01T15:30:00Z",
  "uptime_seconds": 3600,
  "database_connected": true,
  "memory_usage_mb": 45.2,
  "cpu_usage_percent": 12.5,
  "active_connections": 5
}
```

### Status Report Response
```json
{
  "timestamp": "2026-04-01T15:30:00Z",
  "uptime_seconds": 3600,
  "health": {
    "status": "healthy",
    "database_connected": true,
    "memory_usage_mb": 45.2,
    "cpu_usage_percent": 12.5,
    "active_connections": 5
  },
  "requests": {
    "total_requests": 1250,
    "requests_per_second": 15.2,
    "average_response_time": "45ms",
    "last_request": "2026-04-01T15:29:58Z"
  },
  "status_codes": [
    {
      "code": 200,
      "count": 1100,
      "last_seen": "2026-04-01T15:29:58Z",
      "category": "success"
    },
    {
      "code": 404,
      "count": 25,
      "last_seen": "2026-04-01T15:29:45Z",
      "category": "client_error"
    }
  ],
  "top_errors": [
    {
      "code": 404,
      "count": 25,
      "last_seen": "2026-04-01T15:29:45Z",
      "category": "client_error"
    }
  ],
  "performance": {
    "average_response_time_ms": 45,
    "requests_per_second": 15.2,
    "error_rate": 0.02
  }
}
```

## 🔧 Error Types

### Database Errors
```rust
ApiError::Database(sqlx::Error) // HTTP 500
```

### Authentication Errors
```rust
ApiError::Auth(String) // HTTP 401
```

### Validation Errors
```rust
ApiError::Validation(String) // HTTP 400
```

### Not Found Errors
```rust
ApiError::NotFound(String) // HTTP 404
```

### Conflict Errors
```rust
ApiError::Conflict(String) // HTTP 409
```

### Forbidden Errors
```rust
ApiError::Forbidden(String) // HTTP 403
```

### Internal Errors
```rust
ApiError::Internal(String) // HTTP 500
```

### Bad Request Errors
```rust
ApiError::BadRequest(String) // HTTP 400
```

### Unauthorized Errors
```rust
ApiError::Unauthorized(String) // HTTP 401
```

### Service Unavailable Errors
```rust
ApiError::ServiceUnavailable(String) // HTTP 503
```

## 📝 Logging Levels

### Error (ERROR)
- 5xx status codes (server errors)
- Database connection failures
- Critical system errors

### Warn (WARN)
- 4xx status codes (client errors)
- Authentication failures
- Authorization issues

### Info (INFO)
- Successful operations
- Service startup/shutdown
- General operational information

### Debug (DEBUG)
- 2xx/3xx status codes (success/redirect)
- Request/response details
- Performance metrics

## 🛠️ Usage Examples

### Custom Error Handling
```rust
use crate::logging::{ApiError, error_responses};

pub async fn create_user(
    // ... parameters
) -> Result<ApiResponse, ApiError> {
    // Validation
    if request.username.is_empty() {
        return Err(error_responses::validation_error("Username is required"));
    }
    
    // Database operation
    let user = user_service.create_user(request)
        .await
        .map_err(|e| error_responses::database_error(format!("Failed to create user: {}", e)))?;
    
    // Success response
    Ok(ApiResponse::created().data(user))
}
```

### Success Response Builder
```rust
use crate::logging::ApiResponse;

// Simple success
Ok(ApiResponse::success())

// With data
Ok(ApiResponse::success().data(user_list))

// With metadata
Ok(ApiResponse::success()
    .data(server_list)
    .meta("count", server_list.len())
    .meta("page", 1))

// Custom message
Ok(ApiResponse::new()
    .message("Operation completed successfully")
    .data(result))
```

### Monitoring Status
```bash
# Check health status
curl http://127.0.0.1:3000/health

# Get detailed status report
curl http://127.0.0.1:3000/status

# Monitor logs in real-time
tail -f /var/log/game-panel.log
```

## 📊 Metrics Tracked

### Request Metrics
- Total requests processed
- Requests per second
- Average response time
- Last request timestamp

### Status Code Metrics
- Count by status code
- Error rate (4xx + 5xx / total)
- Top error codes
- Last seen timestamps

### System Metrics
- Uptime in seconds
- Memory usage (MB)
- CPU usage (percentage)
- Active connections

### Health Metrics
- Overall health status (healthy/degraded/unhealthy)
- Database connectivity
- Service availability

## 🔍 Monitoring Dashboard

You can create a simple monitoring dashboard by polling the `/status` endpoint:

```javascript
// JavaScript example for real-time monitoring
async function fetchStatus() {
    const response = await fetch('/status');
    const data = await response.json();
    
    // Update dashboard
    updateHealthStatus(data.health);
    updateRequestMetrics(data.requests);
    updateErrorChart(data.status_codes);
    updatePerformanceMetrics(data.performance);
}

// Poll every 5 seconds
setInterval(fetchStatus, 5000);
```

## 🚨 Alerting

The system automatically logs errors at appropriate levels. You can set up alerting based on:

### Error Rate Thresholds
- Error rate > 10%: WARN level
- Error rate > 50%: ERROR level

### Response Time Thresholds
- Average response time > 1s: WARN level
- Average response time > 5s: ERROR level

### System Resource Thresholds
- Memory usage > 80%: WARN level
- CPU usage > 90%: ERROR level

## 🧪 Testing

### Test Error Handling
```bash
# Trigger validation error
curl -X POST http://127.0.0.1:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "", "email": "test@example.com", "password": "123"}'

# Trigger not found error
curl http://127.0.0.1:3000/api/v1/servers/00000000-0000-0000-0000-000000000000

# Check status report
curl http://127.0.0.1:3000/status
```

### Test Performance
```bash
# Load test with multiple requests
for i in {1..100}; do
  curl -s http://127.0.0.1:3000/health > /dev/null
done

# Check metrics
curl http://127.0.0.1:3000/status | jq '.performance'
```

## 📚 Integration

### Frontend Integration
```typescript
interface ApiResponse<T> {
    success: boolean;
    data?: T;
    message?: string;
    meta?: Record<string, any>;
    timestamp: string;
}

interface ApiError {
    success: false;
    error: {
        code: string;
        message: string;
        category: string;
    };
    timestamp: string;
    path: string;
}

// Usage in frontend
async function apiCall<T>(url: string, options?: RequestInit): Promise<T> {
    const response = await fetch(url, options);
    const data = await response.json();
    
    if (!data.success) {
        throw new Error(data.error.message);
    }
    
    return data.data;
}
```

### Monitoring Integration
```python
# Python example for monitoring
import requests
import time

def monitor_api():
    while True:
        try:
            response = requests.get('http://127.0.0.1:3000/status')
            data = response.json()
            
            # Check health status
            if data['health']['status'] != 'healthy':
                send_alert(f"API status: {data['health']['status']}")
            
            # Check error rate
            if data['performance']['error_rate'] > 0.1:
                send_alert(f"High error rate: {data['performance']['error_rate']}")
            
        except Exception as e:
            send_alert(f"Monitoring failed: {e}")
        
        time.sleep(30)
```

## 🔄 Continuous Monitoring

The status monitor runs in the background and updates every 30 seconds:

- Collects system metrics
- Calculates error rates
- Updates health status
- Logs performance data
- Maintains rolling statistics

## 🎯 Best Practices

1. **Use structured error responses** - Always return `Result<ApiResponse, ApiError>`
2. **Log at appropriate levels** - Use the built-in logging system
3. **Monitor error rates** - Set up alerts for high error rates
4. **Track performance** - Monitor response times and throughput
5. **Health checks** - Use `/health` for load balancer health checks
6. **Status reports** - Use `/status` for detailed monitoring

---

**The status logging system provides comprehensive visibility into your API performance and health, making it easy to monitor, debug, and maintain your game server panel backend.**
