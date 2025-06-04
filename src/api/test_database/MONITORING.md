# 📊 Database Monitoring Endpoint Documentation

## 🚀 Overview

The `/db/monitoring` endpoint provides comprehensive database monitoring capabilities for both **local instance information** and **global PostgreSQL statistics**. This endpoint is designed to work seamlessly in both single-instance and multi-replica Docker deployments.

## 🔗 Endpoint Details

- **URL**: `GET /db/monitoring`
- **Purpose**: Real-time database connection monitoring and statistics
- **Response Format**: JSON with standardized structure
- **Authentication**: None (public endpoint)

## 📋 Response Structure

```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "instance_info": { /* Local instance data */ },
    "global_db_stats": { /* Global PostgreSQL stats */ }
  },
  "messages": ["Database monitoring data retrieved"],
  "date": "2025-06-04T21:14:35.871644600+00:00"
}
```

## 🏠 Instance Information (`instance_info`)

This section provides **local information** specific to the current application instance.

### Structure:
```json
"instance_info": {
  "instance_id": "host_DESKTOP-S3AM6FO",
  "local_pools": {
    "active_pools": ["master", "tenant_7cb785743d3949b996c895173cd1a54b"],
    "pool_count": 2
  }
}
```

### Field Descriptions:

#### `instance_id` (String)
- **Purpose**: Unique identifier for this application instance
- **Format Options**:
  - `container_abc123456789` - Docker container ID (first 12 chars)
  - `host_DESKTOP-S3AM6FO` - Server hostname
  - `process_12345` - Process ID (fallback)
- **Use Case**: Distinguish between multiple replicas in deployment

#### `local_pools` (Object)
Information about connection pools managed by this specific instance.

##### `active_pools` (Array of Strings)
- **Purpose**: List of active connection pool identifiers
- **Format**: `["master", "tenant_{uuid}", "tenant_{uuid}_{app}"]`
- **Examples**:
  - `"master"` - Master database schema pool
  - `"tenant_abc123"` - Specific tenant schema pool
  - `"tenant_xyz789_mobile"` - Tenant pool for mobile app
- **Use Case**: Track which schemas this instance is actively connected to

##### `pool_count` (Number)
- **Purpose**: Total number of active connection pools
- **Range**: 0 to N (typically 1-50 depending on tenant usage)
- **Use Case**: Quick assessment of local resource usage

## 🌍 Global Database Statistics (`global_db_stats`)

This section provides **server-wide PostgreSQL statistics** that are the same across all application instances.

### Structure:
```json
"global_db_stats": {
  "connection_summary": { /* Overall connection stats */ },
  "max_connections": { /* Connection limits and usage */ },
  "database_info": { /* Database size information */ },
  "connections_by_database": [ /* Per-database breakdown */ ],
  "connections_by_application": [ /* Per-application breakdown */ ]
}
```

## 📊 Connection Summary (`connection_summary`)

```json
"connection_summary": {
  "total": 16,
  "active": 1,
  "idle": 10,
  "other": 5
}
```

### Field Descriptions:

#### `total` (Number)
- **Purpose**: Total connections to PostgreSQL server
- **Includes**: All connection states (active, idle, etc.)
- **Use Case**: Overall server load assessment

#### `active` (Number)
- **Purpose**: Connections currently executing queries
- **SQL State**: `state = 'active'`
- **Use Case**: Real-time database activity monitoring

#### `idle` (Number)
- **Purpose**: Connections waiting for new commands
- **SQL State**: `state = 'idle'`
- **Use Case**: Connection pool efficiency analysis

#### `other` (Number)
- **Purpose**: Connections in other states
- **Includes**: `idle in transaction`, `fastpath function call`, etc.
- **Calculation**: `total - active - idle`
- **Use Case**: Detect problematic connections (stuck transactions)

## 🔧 Max Connections (`max_connections`)

### Success Response:
```json
"max_connections": {
  "value": 100,
  "usage_percentage": 16,
  "status": "available"
}
```

### Error Response:
```json
"max_connections": {
  "value": null,
  "usage_percentage": null,
  "status": "error",
  "error": "Unable to access PostgreSQL configuration"
}
```

### Field Descriptions:

#### `value` (Number | null)
- **Purpose**: Maximum allowed connections configured in PostgreSQL
- **Source**: `pg_settings.max_connections`
- **Use Case**: Capacity planning and alerting

#### `usage_percentage` (Number | null)
- **Purpose**: Current usage as percentage of maximum
- **Calculation**: `(total_connections / max_connections) * 100`
- **Range**: 0-100+
- **Use Case**: Connection pool sizing and alerts

#### `status` (String)
- **Values**: `"available"` | `"error"`
- **Purpose**: Indicates if max_connections info was retrieved successfully

## 💾 Database Info (`database_info`)

```json
"database_info": {
  "size_mb": 7,
  "size_gb": 0.01
}
```

### Field Descriptions:

#### `size_mb` (Number)
- **Purpose**: Current database size in megabytes
- **Source**: `pg_database_size(current_database())`
- **Use Case**: Storage monitoring and growth tracking

#### `size_gb` (Number)
- **Purpose**: Current database size in gigabytes (rounded to 2 decimals)
- **Calculation**: `size_mb / 1024` (rounded)
- **Use Case**: Human-readable size display

## 🗄️ Connections by Database (`connections_by_database`)

```json
"connections_by_database": [
  {
    "database": "rust_axum_api",
    "connections": 11,
    "percentage": 69
  }
]
```

### Purpose
Shows how connections are distributed across different databases on the PostgreSQL server.

### Field Descriptions:

#### `database` (String)
- **Purpose**: Name of the database
- **Examples**: `"rust_axum_api"`, `"postgres"`, `"analytics_db"`
- **Source**: `pg_stat_activity.datname`

#### `connections` (Number)
- **Purpose**: Number of connections to this specific database
- **Use Case**: Identify which databases are most heavily used

#### `percentage` (Number)
- **Purpose**: Percentage of total connections used by this database
- **Calculation**: `(database_connections / total_connections) * 100`
- **Use Case**: Resource distribution analysis

### What Connections Might Not Appear Here:
- **Administrative connections** without a specific database
- **System connections** (replication, maintenance)
- **Connections** that haven't selected a database yet

## 🔧 Connections by Application (`connections_by_application`)

```json
"connections_by_application": [
  {
    "application": "TablePlus",
    "connections": 1,
    "percentage": 6
  }
]
```

### Purpose
Shows which applications/tools are connected to the database.

### Field Descriptions:

#### `application` (String)
- **Purpose**: Name of the connecting application
- **Examples**: `"TablePlus"`, `"my-axum-project"`, `"pgAdmin"`, `"psql"`
- **Source**: `pg_stat_activity.application_name`

#### `connections` (Number)
- **Purpose**: Number of connections from this application
- **Use Case**: Identify connection sources and potential issues

#### `percentage` (Number)
- **Purpose**: Percentage of total connections from this application
- **Use Case**: Application resource usage analysis

## 🎯 Use Cases & Scenarios

### 1. **Single Instance Monitoring**
- Monitor local connection pools
- Track database growth
- Identify connection bottlenecks

### 2. **Multi-Replica Deployment**
- **Local Data**: Each replica shows its own pools
- **Global Data**: All replicas show same PostgreSQL stats
- **Instance Identification**: Distinguish between containers

### 3. **Performance Optimization**
- **High `usage_percentage`**: Need to increase max_connections or optimize pools
- **Many `idle` connections**: Pool sizing might be too large
- **High `other` connections**: Potential stuck transactions

### 4. **Troubleshooting**
- **Missing applications**: Expected apps not showing in connections_by_application
- **Unexpected databases**: Unknown databases appearing in connections_by_database
- **Connection leaks**: Growing total_connections without corresponding app activity

## 🚨 Common Alert Scenarios

### Connection Usage Alerts
```bash
# High usage (>80%)
usage_percentage > 80

# Many idle connections (>75% of total)
(idle_connections / total_connections) > 0.75

# Stuck transactions (>10% other connections)
(other_connections / total_connections) > 0.10
```

### Growth Alerts
```bash
# Database size growth
size_gb > threshold

# Rapid connection growth
total_connections > expected_baseline
```

## 🔄 Integration Examples

### Load Balancer Health Check
```bash
curl -f http://api-instance/db/monitoring
# Check if response.data.global_db_stats.max_connections.status == "available"
```

### Monitoring Dashboard
```javascript
// Parse response for dashboard
const response = await fetch('/db/monitoring');
const data = await response.json();

const metrics = {
  instanceId: data.data.instance_info.instance_id,
  localPools: data.data.instance_info.local_pools.pool_count,
  globalConnections: data.data.global_db_stats.connection_summary.total,
  dbUsage: data.data.global_db_stats.max_connections.usage_percentage
};
```

### Docker Compose Health Check
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/db/monitoring"]
  interval: 30s
  timeout: 10s
  retries: 3
```

## 🛠️ Technical Implementation

### Data Sources
- **Local Pools**: In-memory HashMap in `DatabaseService`
- **Global Stats**: Direct PostgreSQL queries to `pg_stat_activity`
- **Instance ID**: Environment variables, hostname, or process ID

### PostgreSQL Queries Used
```sql
-- Total connections by state
SELECT count(*) FROM pg_stat_activity WHERE state = 'active';
SELECT count(*) FROM pg_stat_activity WHERE state = 'idle';
SELECT count(*) FROM pg_stat_activity;

-- Connections by database
SELECT datname, count(*) as conn_count 
FROM pg_stat_activity 
WHERE datname IS NOT NULL 
GROUP BY datname 
ORDER BY conn_count DESC;

-- Connections by application
SELECT COALESCE(application_name, 'unknown') as app_name, count(*) 
FROM pg_stat_activity 
WHERE application_name IS NOT NULL AND application_name != ''
GROUP BY application_name;

-- Database size
SELECT pg_database_size(current_database());

-- Max connections setting
SELECT setting::int FROM pg_settings WHERE name = 'max_connections';
```

## 📈 Performance Considerations

- **Low Overhead**: Queries are lightweight and optimized
- **Caching**: Results are computed on-demand (no caching)
- **Scalability**: Performance scales with PostgreSQL, not application instances
- **Error Handling**: Graceful degradation if PostgreSQL queries fail

---

## 🎉 Quick Start

```bash
# Test the endpoint
curl http://localhost:3000/db/monitoring | jq

# Check specific metrics
curl -s http://localhost:3000/db/monitoring | jq '.data.global_db_stats.connection_summary'

# Monitor in real-time
watch -n 5 'curl -s http://localhost:3000/db/monitoring | jq ".data.global_db_stats.connection_summary"'
```

This endpoint provides everything you need to monitor your database connections in both development and production environments! 🚀 