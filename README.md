# REC (Really Easy Config)

## Overview

REL is a typed configuration language that combines YAML-like readability with strict type safety and validation. It uses curly braces for structure instead of indentation.

## File Extension

`.rel`

## Basic Syntax

### Key-Value Pairs

```rel
{
  name: "My Application"
  port: 8080
  debug: true
}
```

### Nested Objects

```rel
{
  server: {
    host: "0.0.0.0"
    port: 8080
  }
  database: {
    url: "postgresql://localhost/mydb"
    pool_size: 10
  }
}
```

### Arrays

```rel
{
  allowed_origins: ["http://localhost:3000", "https://example.com"]
  ports: [8080, 8081, 8082]
}
```

## Type System

### Basic Types

- `string`: Text values in quotes
- `int`: Integer numbers
- `float`: Floating point numbers
- `bool`: `true` or `false`
- `null`: Null value

### Extended Types

#### Enums

Simple enums (unit variants):
```rel
@enum LogLevel {
  DEBUG
  INFO
  WARN
  ERROR
}

{
  log_level: LogLevel.INFO
}
```

Enums with struct variants:
```rel
@enum DatabaseConnection {
  Postgres { host: string, port: int, database: string, ssl: bool }
  MySQL { host: string, port: int, database: string }
  SQLite { file_path: string, read_only: bool }
  InMemory
}

@enum Command {
  Start { port: int, workers: int }
  Stop
  Restart { delay_seconds: int, graceful: bool }
  Status
}

{
  primary_db: DatabaseConnection.Postgres {
    host: "localhost"
    port: 5432
    database: "myapp"
    ssl: true
  }
  
  fallback_db: DatabaseConnection.SQLite {
    file_path: "/tmp/backup.db"
    read_only: false
  }
  
  startup_command: Command.Start {
    port: 8080
    workers: 4
  }
}
```

Enums with tuple variants:
```rel
@enum CacheStrategy {
  NoCache
  FixedTTL(int)  // TTL in seconds
  SlidingWindow(int, int)  // (window_size, ttl)
  Custom { strategy: string, params: [string] }
}

{
  cache: CacheStrategy.FixedTTL(300)
  backup_cache: CacheStrategy.SlidingWindow(100, 600)
}
```

#### URLs

```rel
{
  api_endpoint: url("https://api.example.com")
  webhook: url("http://localhost:3000/webhook")
}
```

#### IPv4 Socket Addresses

```rel
{
  bind_address: socket("127.0.0.1:8080")
  redis_server: socket("192.168.1.100:6379")
}
```

#### Base58 Pubkeys (Solana)

```rel
{
  wallet_address: pubkey("11111111111111111111111111111111")
  program_id: pubkey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
}
```

## Type Annotations

```rel
@type ServerConfig {
  host: string
  port: int
  ssl_enabled: bool
  ssl_cert?: string  // Optional field
}

{
  server: ServerConfig {
    host: "localhost"
    port: 443
    ssl_enabled: true
    ssl_cert: "/path/to/cert.pem"
  }
}
```

## Include Statements

```rel
#include "common/database.rel"
#include "secrets/api_keys.rel"

{
  app_name: "MyService"
  // Included configs are merged here
}
```

## Comments

```rel
{
  // Single line comment
  name: "MyApp"
  
  /* Multi-line
     comment */
  port: 8080
}
```

## Complete Example

```rel
#include "common/base.rel"

@enum Environment {
  DEVELOPMENT
  STAGING
  PRODUCTION
}

@type DatabaseConfig {
  url: string
  max_connections: int
  timeout_ms: int
}

@type ServerConfig {
  bind_address: socket
  environment: Environment
  allowed_origins: [url]
  admin_pubkey: pubkey
}

{
  server: ServerConfig {
    bind_address: socket("0.0.0.0:8080")
    environment: Environment.DEVELOPMENT
    allowed_origins: [
      url("http://localhost:3000"),
      url("https://app.example.com")
    ]
    admin_pubkey: pubkey("DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy")
  }
  
  database: DatabaseConfig {
    url: "postgresql://user:pass@localhost/mydb"
    max_connections: 20
    timeout_ms: 5000
  }
  
  features: {
    rate_limiting: true
    cache_enabled: true
    cache_ttl_seconds: 300
  }
}
```

## Validation Rules

1. **Type Checking**: All values must match their declared types
2. **Enum Values**: Only defined enum variants are allowed
3. **URL Validation**: URLs must be valid HTTP/HTTPS URLs
4. **Socket Validation**: Must be valid IPv4:port format
5. **Pubkey Validation**: Must be valid Base58 encoded 32-byte ed25519 public keys
6. **Required Fields**: Non-optional fields must be present
7. **No Duplicate Keys**: Keys must be unique within their scope