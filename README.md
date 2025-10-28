# schema

**Like `serde`, but for structure instead of serialization.**

Turn Rust types into JSON schemas that describe their shape. Works with Claude AI tools, OpenAPI specs, or anything else that needs to understand your data structure.

```rust
#[derive(Schema)]
struct User { name: String, age: u32 }

User::schema() // → JSON schema describing the structure
```

## Usage

```rust
use schema::Schema;

#[derive(Schema)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

let schema = User::schema();
```

## Crates

- **schema** - Core derive macro
- **schema-anthropic** - Anthropic Claude tool schemas
- **schema-openapi** - OpenAPI 3.0 specs

## Examples

### Anthropic Tools

```rust
use schema::Schema;
use schema_anthropic::to_tool_schema;

#[derive(Schema)]
struct SearchFiles {
    /// Query to search for
    query: String,
    /// Maximum results to return
    limit: Option<u32>,
}

let tool = to_tool_schema::<SearchFiles>("search_files");
// Use with Claude API
```

### OpenAPI Specs

```rust
use schema::Schema;
use schema_openapi::to_openapi_schema;

#[derive(Schema)]
struct CreateUser {
    username: String,
    email: String,
}

let spec = to_openapi_schema::<CreateUser>();
// Returns OpenAPI 3.0 schema object
```

### Enums

```rust
#[derive(Schema)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Schema)]
enum Message {
    Text { content: String },
    Image { url: String, width: u32 },
}
```

## Features

- Derives from Rust types
- Doc comments → descriptions
- `Option<T>` → optional fields
- Enums → string enums or tagged unions
- Nested structs supported
- `#[schema(skip)]` to skip fields

## Installation

```toml
[dependencies]
schema = { git = "https://github.com/andrewgazelka/schema" }
schema-anthropic = { git = "https://github.com/andrewgazelka/schema" }
schema-openapi = { git = "https://github.com/andrewgazelka/schema" }
```

## License

MIT
