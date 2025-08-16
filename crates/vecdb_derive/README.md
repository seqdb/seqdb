# vecdb_derive

Procedural macros for [vecdb](../vecdb) that enable custom types to work with compressed storage.

## What is vecdb_derive?

This crate provides derive macros that automatically implement compression traits for custom wrapper types, allowing them to be used seamlessly with vecdb's compressed storage variants.

## Features

- **Automatic trait implementation**: Generates `StoredCompressed` for wrapper types
- **Zero-cost abstractions**: Wrappers have the same compression characteristics as inner types
- **Generic support**: Works with generic types and proper trait bounds
- **Type safety**: Compile-time guarantees for compression compatibility

## Derive Macros

### `#[derive(StoredCompressed)]`

Automatically implements compression traits for single-field tuple structs.

**Requirements:**
- Must be a tuple struct with exactly one field
- The inner type must implement `StoredCompressed`

## Usage

### Basic Wrapper Types

```rust
use vecdb_derive::StoredCompressed;
use vecdb::{CompressedVec, Database, Version};

// Type-safe wrappers around numeric types
#[derive(StoredCompressed, Debug, Clone, Copy, PartialEq)]
struct UserId(u32);

#[derive(StoredCompressed, Debug, Clone, Copy, PartialEq)]
struct Score(f64);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("data")?;
    
    // Use custom types in compressed vectors
    let mut scores: CompressedVec<UserId, Score> = 
        CompressedVec::forced_import(&db, "user_scores", Version::TWO)?;
    
    scores.push(Score(95.5));
    scores.push(Score(87.2));
    scores.flush()?;
    
    Ok(())
}
```

### Generic Wrapper Types

```rust
use vecdb_derive::StoredCompressed;

// Generic wrapper preserves compression characteristics
#[derive(StoredCompressed, Debug, Clone, Copy, PartialEq)]
struct Metric<T>(T);

// Can be used with any StoredCompressed type
type Temperature = Metric<f32>;
type Count = Metric<u64>;
```

### Real-World Example

```rust
use vecdb_derive::StoredCompressed;
use vecdb::{CompressedVec, Database, Version};

#[derive(StoredCompressed, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Timestamp(u64);

#[derive(StoredCompressed, Debug, Clone, Copy, PartialEq)]
struct SensorReading(f32);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("sensors")?;
    
    let mut readings: CompressedVec<Timestamp, SensorReading> = 
        CompressedVec::forced_import(&db, "temperature", Version::TWO)?;
    
    let now = Timestamp(1640995200);
    readings.push(SensorReading(23.5));
    readings.flush()?;
    
    Ok(())
}
```

## Generated Code

For a simple wrapper:
```rust
#[derive(StoredCompressed)]
struct UserId(u32);
```

The macro generates:
```rust
impl ::vecdb::TransparentStoredCompressed<u32> for UserId {}

impl StoredCompressed for UserId {
    type NumberType = u32;
}
```

For generic types:
```rust
#[derive(StoredCompressed)]
struct Wrapper<T>(T);
```

The macro generates:
```rust
impl<T> ::vecdb::TransparentStoredCompressed<T::NumberType> for Wrapper<T> 
where T: StoredCompressed {}

impl<T> StoredCompressed for Wrapper<T> 
where T: StoredCompressed {
    type NumberType = T::NumberType;
}
```

## Error Messages

Clear error messages for common mistakes:
- **Wrong structure**: "StoredCompressed can only be derived for single-field tuple structs"
- Only tuple structs with exactly one field are supported

## Limitations

- Only works with single-field tuple structs
- Inner type must implement `StoredCompressed`
- Does not work with enums, regular structs, or unit structs

## Benefits

1. **Type Safety**: Prevent mixing incompatible data types
2. **Zero Cost**: No runtime overhead compared to raw types
3. **Compression**: Maintains all compression benefits of the inner type
4. **Integration**: Works seamlessly with all vecdb storage variants

## When to Use

Use `#[derive(StoredCompressed)]` to:
- Create domain-specific wrapper types
- Add type safety to numeric identifiers
- Build APIs that prevent value confusion
- Maintain compression efficiency with custom types

## Compatibility

Derived types work with all vecdb storage variants:
- `CompressedVec<I, T>`: Compressed storage
- `RawVec<I, T>`: Uncompressed storage
- `ComputedVec`: Derived data storage