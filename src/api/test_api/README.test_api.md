# Test API Module

Specialized endpoints for validating and testing all IC360 API middlewares, including error handling, timeouts, body size limits, and standard responses.

## Purpose

These endpoints are designed to:
- Validate middleware stack during development
- Test timeout and limit configurations
- Verify error handling and consistent responses
- Debug configuration issues
- Perform basic system health checks

## Endpoints

### 1. Hello - `GET /hello`
Basic endpoint that returns version information.

```bash
curl http://localhost:3000/hello
```

**Response:**
```json
{
  "status": "OK",
  "code": 200,
  "data": {"version": "1.0.0"},
  "messages": ["Service started successfully"],
  "date": "2024-01-15T10:30:00.123Z"
}
```

### 2. Status - `GET /status`
System status with environment information.

```bash
curl http://localhost:3000/status
```

**Response:**
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "version": "1.0.0",
    "status": "healthy",
    "environment": "development"
  },
  "messages": ["API is running successfully"],
  "date": "2024-01-15T10:30:00.123Z"
}
```

### 3. Timeout Test - `GET /timeout`
Deliberately exceeds configured timeout to test timeout middleware.

- **Configuration**: `DEFAULT_TIMEOUT_SECONDS` (default 30s)
- **Expected Result**: 408 Request Timeout

```bash
time curl http://localhost:3000/timeout
# Should timeout after ~30 seconds with 408 status
```

### 4. Error Test - `GET /error`
Deliberately returns a 500 error to test error handling.

```bash
curl http://localhost:3000/error
# Returns 500 Internal Server Error
```

### 5. Not Found Test - `GET /not-found`
Deliberately returns a 404 error to test not found handling.

```bash
curl http://localhost:3000/not-found
# Returns 404 Not Found
```

### 6. Body Size Test - `POST /body-size`
Tests request body size limits.

```bash
# Test with small body (should succeed)
curl -X POST http://localhost:3000/body-size \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'

# Test with large body (should fail with 413)
curl -X POST http://localhost:3000/body-size \
  -H "Content-Type: application/json" \
  -d '{"large": "'$(head -c 3000000 /dev/zero | tr '\0' 'a')'"}'
```

## Usage

### Development Testing
```bash
# Quick health check
curl http://localhost:3000/hello

# Full middleware validation
curl http://localhost:3000/status
curl http://localhost:3000/error
curl http://localhost:3000/not-found
curl -X POST http://localhost:3000/body-size -d '{"test":"data"}'
```

### Configuration Validation
```bash
# Test timeout settings
time curl http://localhost:3000/timeout

# Test body size limits
curl -X POST http://localhost:3000/body-size \
  -d '{"data":"'$(head -c 1000000 /dev/zero | tr '\0' 'a')'"}'
```

### Load Balancer Health Checks
```bash
# Simple health check
curl -f http://localhost:3000/status && echo "✅ API healthy"

# Version check
curl -s http://localhost:3000/hello | jq '.data.version'
```

## Error Responses

All error responses follow the standard format:

```json
{
  "status": "ERROR_TYPE",
  "code": 4XX_OR_5XX,
  "data": {"error_details": "..."},
  "messages": ["Error description"],
  "date": "ISO_TIMESTAMP"
}
```

## Features

- **Middleware Validation**: Tests all middleware layers
- **Configuration Testing**: Validates timeout and size limits
- **Error Simulation**: Controlled error scenarios
- **Health Monitoring**: System status endpoints
- **Standard Responses**: Consistent JSON format across all endpoints 