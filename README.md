# Rust PostgreSQL API

A REST API built with Rust, Actix-web, and PostgreSQL that allows dynamic table querying with flexible filtering, pagination, sorting, and multiple filter support.

## Features

- ✅ Dynamic table querying
- ✅ Flexible filtering with operators: `=`, `!=`, `>`, `<`, `>=`, `<=`
- ✅ Multiple filters (AND logic)
- ✅ Pagination support
- ✅ Sorting (ascending/descending)
- ✅ URL encoding support for spaces and special characters
- ✅ Docker containerization
- ✅ PostgreSQL database
- ✅ Health check endpoint
- ✅ Total count for pagination

## Project Structure

```
.
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── init.sql
├── .env
└── src/
    └── main.rs
```

## Quick Start

### Prerequisites

- Docker and Docker Compose installed

### Running the Application

1. Build and start the services:

```bash
docker-compose up --build
```

2. The API will be available at `http://localhost:8080`

## API Endpoints

### Health Check

```bash
curl http://localhost:8080/health
```

### Query All Records (with Pagination)

**Format:** `/{table_name}?page=1&page_size=10&sort=column&order=asc`

**Examples:**

```bash
# Get all users (default: page 1, 100 records)
curl http://localhost:8080/users

# Get page 2 with 10 records per page
curl "http://localhost:8080/users?page=2&page_size=10"

# Sort by name ascending
curl "http://localhost:8080/users?sort=name&order=asc"

# Sort by id descending with pagination
curl "http://localhost:8080/sample_data?page=1&page_size=5&sort=id&order=desc"
```

### Query with Single Filter

**Format:** `/{table_name}/{column}{operator}{value}?page=1&page_size=10&sort=column&order=asc`

**Examples:**

```bash
# Filter by date
curl http://localhost:8080/sample_data/date=2025-10-10

# Filter by ID greater than or equal to 3
curl http://localhost:8080/sample_data/id>=3

# Filter by value greater than 150 with sorting
curl "http://localhost:8080/sample_data/value>150?sort=value&order=desc"

# Filter by name with spaces (URL encoded automatically by curl -G --data-urlencode)
curl -G --data-urlencode "name=Alice Johnson" http://localhost:8080/users/name=Alice%20Johnson

# Or manually encode spaces as %20
curl "http://localhost:8080/users/name=Alice%20Johnson"

# Filter users by id with pagination
curl "http://localhost:8080/users/id>1?page=1&page_size=2"
```

### Query with Multiple Filters (AND logic)

**Format:** `/{table_name}/{filter1}&{filter2}&{filter3}?page=1&page_size=10`

**Examples:**

```bash
# Multiple conditions: date AND value
curl "http://localhost:8080/sample_data/date=2025-10-10&value>100"

# Multiple conditions with pagination
curl "http://localhost:8080/sample_data/id>1&value<200?page=1&page_size=5"

# Filter users by multiple conditions
curl "http://localhost:8080/users/id>1&name!=Bob%20Smith"
```

### Response Format

```json
{
  "count": 2,
  "data": [
    {
      "id": 1,
      "date": "2025-10-10",
      "value": 100.50
    },
    {
      "id": 2,
      "date": "2025-10-10",
      "value": 200.75
    }
  ],
  "page": 1,
  "page_size": 100,
  "total_count": 2
}
```

## Query Parameters

### Pagination

- `page` (optional, default: 1) - Page number (starts from 1)
- `page_size` (optional, default: 100, max: 1000) - Number of records per page

### Sorting

- `sort` (optional) - Column name to sort by
- `order` (optional, default: "asc") - Sort order: `asc` or `desc`

## Supported Operators

- `=` - Equal to
- `!=` - Not equal to
- `>` - Greater than
- `<` - Less than
- `>=` - Greater than or equal to
- `<=` - Less than or equal to

## Handling Spaces in Values

The API supports URL encoding for values with spaces. You have several options:

### Option 1: Use curl with automatic encoding

```bash
curl -G --data-urlencode "filter" "http://localhost:8080/users/name=Alice Johnson"
```

### Option 2: Manually encode spaces as %20

```bash
curl "http://localhost:8080/users/name=Alice%20Johnson"
```

### Option 3: Use + for spaces (also supported)

```bash
curl "http://localhost:8080/users/name=Alice+Johnson"
```

## Real-World Examples

### Example 1: Paginated User List

```bash
# Get first 10 users sorted by name
curl "http://localhost:8080/users?page=1&page_size=10&sort=name&order=asc"
```

### Example 2: Find High-Value Records

```bash
# Get all records with value > 150, sorted by value descending
curl "http://localhost:8080/sample_data/value>150?sort=value&order=desc"
```

### Example 3: Date Range with Multiple Filters

```bash
# Records from specific date with value constraints
curl "http://localhost:8080/sample_data/date>=2025-10-10&value<300"
```

### Example 4: Search User by Name

```bash
# Find user "Alice Johnson"
curl "http://localhost:8080/users/name=Alice%20Johnson"
```

## Database

The `init.sql` file creates two sample tables:

1. **sample_data**: Contains id, date, and value columns
2. **users**: Contains id, name, email, and created_at columns

You can modify `init.sql` to add your own tables and data.

## Development

### Running Locally (without Docker)

1. Start PostgreSQL:

```bash
docker run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  postgres:15-alpine
```

2. Apply init.sql:

```bash
docker exec -i postgres psql -U postgres -d testdb < init.sql
```

3. Run the application:

```bash
cargo run
```

### Stopping the Services

```bash
docker-compose down
```

### Removing Volumes

```bash
docker-compose down -v
```

## Performance Tips

1. **Pagination**: Always use pagination for large datasets
2. **Indexes**: Add database indexes on frequently filtered columns
3. **Page Size**: Use smaller page sizes (10-50) for better performance
4. **Sorting**: Sort by indexed columns when possible

## Security Features

- Table and column names are sanitized to prevent SQL injection
- Query values are parameterized
- Only alphanumeric characters and underscores allowed in table/column names
- Maximum page size limit (1000 records)

## Logs

View API logs:

```bash
docker logs -f rust_api
```

View PostgreSQL logs:

```bash
docker logs -f postgres_db
```

## Error Handling

The API returns appropriate HTTP status codes:

- `200 OK` - Successful query
- `400 Bad Request` - Invalid parameters or filters
- `500 Internal Server Error` - Database errors

Error response format:

```json
{
  "error": "Error message here"
}
```