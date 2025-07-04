# Tenant Management API

Complete tenant management endpoints for IC360 API using **single schema with Row-Level Security** for unlimited scalability.

## Architecture

### Single Schema + RLS Design
```
Database: ic360_db
└── Schema: tenants
    └── Table: tenants
        ├── id (UUID, PK)
        ├── name (VARCHAR, unique)
        ├── settings (JSONB)
        ├── is_active (BOOLEAN)
        ├── created_at (TIMESTAMPTZ)
        └── updated_at (TIMESTAMPTZ)
```

### Key Benefits
- **Unlimited Scalability**: No PostgreSQL schema limits
- **Consistent Performance**: Independent of tenant count
- **Simplified Management**: Single backup and migration path
- **Efficient Cross-Tenant Queries**: Easy reporting and analytics
- **Row-Level Security**: Data isolation per tenant

## Endpoints

### 1. Create Tenant - `POST /tenants`
Creates a new tenant in the system.

```bash
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "my_company"}'
```

**Request Body:**
```json
{
  "tenant_name": "tenant_name"
}
```

**Validation Rules:**
- Not empty
- Maximum 100 characters
- Alphanumeric, hyphens (`-`), and underscores (`_`) only
- Auto-normalized to lowercase
- Leading/trailing spaces removed

**Success Response (201 Created):**
```json
{
  "status": "Created",
  "code": 201,
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "my_company",
    "settings": {},
    "is_active": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  },
  "messages": ["Tenant created successfully"],
  "date": "2024-01-15T10:30:00.123Z"
}
```

**Error Response (400 Bad Request):**
```json
{
  "status": "Bad Request",
  "code": 400,
  "data": {
    "error": "tenant_name cannot be empty"
  },
  "messages": ["Invalid tenant name provided"],
  "date": "2024-01-15T10:30:00.123Z"
}
```

### 2. List Tenants - `GET /tenants`
Retrieves all registered tenants.

```bash
curl http://localhost:3000/tenants
```

**Success Response (200 OK):**
```json
{
  "status": "OK",
  "code": 200,
  "data": {
    "tenants": [
      {
        "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "name": "company_alpha",
        "settings": {},
        "is_active": true,
        "created_at": "2024-01-10T08:00:00Z",
        "updated_at": "2024-01-10T08:00:00Z"
      },
      {
        "id": "b2c3d4e5-f6g7-8901-bcde-f12345678901",
        "name": "startup_beta",
        "settings": {},
        "is_active": true,
        "created_at": "2024-01-12T14:30:00Z",
        "updated_at": "2024-01-12T14:30:00Z"
      }
    ],
    "count": 2
  },
  "messages": ["Tenants retrieved successfully"],
  "date": "2024-01-15T10:30:00.123Z"
}
```

## Usage Examples

### Basic Workflow
```bash
# Create tenant
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "new_company"}'

# List all tenants
curl http://localhost:3000/tenants

# Get tenant count
curl -s http://localhost:3000/tenants | jq '.data.count'
```

### Validation Testing
```bash
# Empty name (Error 400)
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": ""}'

# Invalid characters (Error 400)
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "company@invalid"}'

# Auto-normalization (Success)
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "  COMPANY_NAME  "}'
# Result: name becomes "company_name"
```

### Advanced Usage
```bash
# Create tenant with specific settings
curl -X POST http://localhost:3000/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_name": "enterprise_corp",
    "settings": {
      "plan": "enterprise",
      "features": ["analytics", "api_access"],
      "limits": {"users": 1000}
    }
  }'

# Filter active tenants
curl -s http://localhost:3000/tenants | jq '.data.tenants[] | select(.is_active == true)'

# Get tenant by name
curl -s http://localhost:3000/tenants | jq '.data.tenants[] | select(.name == "my_company")'
```

## Error Handling

### Common Error Scenarios
- **400 Bad Request**: Invalid tenant name format
- **409 Conflict**: Tenant name already exists
- **500 Internal Server Error**: Database connection issues

### Error Response Format
```json
{
  "status": "ERROR_TYPE",
  "code": 4XX_OR_5XX,
  "data": {
    "error": "error_description",
    "details": "additional_info"
  },
  "messages": ["User-friendly error message"],
  "date": "ISO_TIMESTAMP"
}
```

## Features

- **Tenant Creation**: Validated tenant registration
- **Tenant Listing**: Complete tenant inventory
- **Name Validation**: Format and uniqueness checking
- **Auto-Normalization**: Consistent naming conventions
- **Settings Support**: Flexible JSON configuration
- **Active Status**: Tenant lifecycle management
- **Row-Level Security**: Data isolation per tenant
- **Unlimited Scaling**: No schema constraints 