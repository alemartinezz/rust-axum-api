# Hola

### Example `.env`

```bash
# Application Environment Settings
# Use "development", "staging", or "production" depending on your deployment.
ENVIRONMENT=development

# Server configuration
HOST=127.0.0.1
PORT=3000
PROTOCOL=http

# Request configuration
# Maximum allowed request body size in bytes (default: 2MB)
MAX_REQUEST_BODY_SIZE=2097152
# Default timeout in seconds for each request
DEFAULT_TIMEOUT_SECONDS=3

# Database configuration
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=postgres
```

### Run in auto reload
```bash
systemfd --no-pid -s http::3000 -- cargo watch -x run
```