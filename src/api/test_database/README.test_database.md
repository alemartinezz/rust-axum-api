# Database Monitoring Endpoints

Comprehensive monitoring endpoints for IC360 API's database architecture using **single schema with Row-Level Security**.

## Architecture

### Before (Schema-per-tenant)
```
Database: ic360_db
├── Schema: master (metadata)
├── Schema: tenant_uuid1 (tenant 1 data)
├── Schema: tenant_uuid2 (tenant 2 data)
└── Schema: tenant_uuid3 (tenant 3 data)
```

### Now (Single schema + RLS)
```
Database: ic360_db
├── Schema: tenants (all tenants with RLS)
│   └── Table: tenants (all tenant records)
├── Schema: master (global functions)
└── Schema: public (standard functions)
```

## Endpoints

### 1. Health Check - `GET /db/health`
Basic database connectivity verification.

```bash
curl http://localhost:3000/db/health
```

**Response:**
```json
{
  "status": "OK",
  "code": 200,
  "data": {"database": "connected"},
  "messages": ["Database connection healthy"],
  "date": "2024-01-15T10:30:00Z"
}
```

### 2. General Monitoring - `GET /db/monitoring`
Global PostgreSQL monitoring (connections, applications, databases).

```bash
curl http://localhost:3000/db/monitoring
```

**Response:**
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "postgresql_cluster": {
      "server": {
        "limits": {
          "max_connections": {
            "value": 100,
            "usage_percentage": 25,
            "status": "available"
          }
        },
        "connections": {
          "total": 25,
          "breakdown": {
            "active": 5,
            "idle": 18,
            "other": 2
          }
        }
      },
      "databases": [
        {
          "ic360_db": {
            "connections": 20,
            "percentage_of_server": 80,
            "size": {
              "size_mb": 45.2,
              "size_gb": 0.04
            },
            "applications": [
              {
                "rust-axum-api": {
                  "connections": 15,
                  "percentage_of_database": 75,
                  "instance_metadata": {
                    "instance_id": "container_abc123def456",
                    "local_connection_pools": {
                      "active_pools": ["tenants", "master"],
                      "pool_count": 2
                    }
                  }
                }
              }
            ]
          }
        }
      ]
    }
  }
}
```

### 3. Tenant Monitoring - `GET /db/tenants/monitoring`
Specific tenant monitoring for single schema architecture.

```bash
curl http://localhost:3000/db/tenants/monitoring
```

**Response:**
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "tenant_architecture": "single_schema_with_rls",
    "schema": "tenants",
    "statistics": {
      "tenant_counts": {
        "total": 157,
        "active": 142,
        "inactive": 15,
        "activity_rate_percentage": 90
      },
      "growth_metrics": {
        "new_tenants_last_7_days": 12,
        "new_tenants_last_30_days": 47,
        "daily_creation_history": [
          {"date": "2024-01-15", "count": 3},
          {"date": "2024-01-14", "count": 5},
          {"date": "2024-01-13", "count": 2}
        ]
      },
      "tenant_timeline": {
        "oldest_tenant": {
          "name": "empresa_alpha",
          "created_at": "2023-06-01T10:00:00Z"
        },
        "newest_tenant": {
          "name": "startup_beta",
          "created_at": "2024-01-15T14:30:00Z"
        }
      },
      "storage_metrics": {
        "table_size_mb": 2.45,
        "table_size_bytes": 2568192,
        "indexes": [
          {
            "name": "idx_tenants_name",
            "size_mb": 0.15,
            "definition": "CREATE INDEX idx_tenants_name ON tenants.tenants(name)"
          },
          {
            "name": "idx_tenants_active",
            "size_mb": 0.08,
            "definition": "CREATE INDEX idx_tenants_active ON tenants.tenants(is_active)"
          }
        ]
      },
      "security": {
        "row_level_security_enabled": true,
        "architecture": "single_schema_with_rls",
        "isolation_method": "row_level_security"
      }
    }
  }
}
```

### 4. Schema Monitoring - `GET /db/schema/monitoring`
Specific monitoring for the "tenants" schema.

```bash
curl http://localhost:3000/db/schema/monitoring
```

**Response:**
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "schema_name": "tenants",
    "statistics": {
      "table_count": 1,
      "total_size_mb": 2.45,
      "total_size_bytes": 2568192,
      "tables": [
        {
          "name": "tenants",
          "size_mb": 2.45,
          "size_bytes": 2568192,
          "row_count": 157,
          "avg_row_size_bytes": 16358,
          "indexes": [
            {
              "name": "idx_tenants_name",
              "size_mb": 0.15,
              "definition": "CREATE INDEX idx_tenants_name ON tenants.tenants(name)"
            },
            {
              "name": "idx_tenants_active",
              "size_mb": 0.08,
              "definition": "CREATE INDEX idx_tenants_active ON tenants.tenants(is_active)"
            }
          ]
        }
      ],
      "functions": [
        {
          "name": "update_updated_at_column",
          "definition": "CREATE OR REPLACE FUNCTION update_updated_at_column()..."
        }
      ],
      "triggers": [
        {
          "name": "update_tenants_updated_at",
          "table": "tenants",
          "function": "update_updated_at_column"
        }
      ]
    }
  }
}
```

## Usage

### Development Monitoring
```bash
# Quick health check
curl http://localhost:3000/db/health

# Full database overview
curl http://localhost:3000/db/monitoring

# Tenant-specific metrics
curl http://localhost:3000/db/tenants/monitoring

# Schema details
curl http://localhost:3000/db/schema/monitoring
```

### Production Monitoring
```bash
# Health check for load balancer
curl -f http://localhost:3000/db/health && echo "✅ Database healthy"

# Monitor tenant growth
curl -s http://localhost:3000/db/tenants/monitoring | jq '.data.statistics.growth_metrics'

# Check storage usage
curl -s http://localhost:3000/db/schema/monitoring | jq '.data.statistics.total_size_mb'
```

### Troubleshooting
```bash
# Check connection status
curl http://localhost:3000/db/health

# Monitor active connections
curl -s http://localhost:3000/db/monitoring | jq '.data.postgresql_cluster.server.connections'

# Verify RLS is enabled
curl -s http://localhost:3000/db/tenants/monitoring | jq '.data.statistics.security.row_level_security_enabled'
```

## Features

- **Health Monitoring**: Database connectivity verification
- **Global Statistics**: PostgreSQL server-wide metrics
- **Tenant Analytics**: Growth, storage, and security metrics
- **Schema Details**: Table sizes, indexes, and functions
- **Security Validation**: RLS policy verification
- **Performance Metrics**: Connection and storage monitoring 