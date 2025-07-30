# REC (Really Easy Config)

## Overview

REC (or "Really Easy Config" for short) is a typed configuration language that combines emulates the brevity of YAML, the structure of JSON, and the strict type safety + validation you'd get with a modern programming lanugage like Rust.

## File Extension

`.rec`

## Basic Syntax

### Key-Value Pairs

```rec
{
  name: "My Application"
  port: 8080
  debug: true
}
```

### Arrays

```rec
{
  allowed_origins: ["http://localhost:3000", "https://example.com"]
  ports: [8080, 8081, 8082]
}
```

## Type System

### Core Primitives

- `string`: Text values in quotes
- `int`: Integer numbers
- `float`: Floating point numbers
- `bool`: `true` or `false`
- `null`: Null value

### Extended Primitives

#### Enums

Unit enums:
```rec
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

Struct Enums:
```rec
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

Tuple Enums
```rec
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

#### HTTP/HTTPS URLs

```rec
{
  api_endpoint: url("https://api.example.com")
  webhook: url("http://localhost:3000/webhook")
}
```

#### IPv4 Sockets

```rec
{
  bind_address: socket("127.0.0.1:8080")
  redis_server: socket("192.168.1.100:6379")
}
```

#### ed25519 Pubkeys (Base58)

```rec
{
  wallet_address: pubkey("11111111111111111111111111111111")
  program_id: pubkey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
}
```

## Types

```rec
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

## Include Statements (external files)

```rec
#include "common/database.rec"
#include "secrets/api_keys.rec"

{
  app_name: "MyService"
  // Included configs are merged here
}
```

## Example

```rec
#include "common/base.rec"

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
