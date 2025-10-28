# vecdb

A K.I.S.S. index-value storage engine that provides persistent, type-safe vector storage with compression and computation capabilities.

## What is vecdb?

vecdb is a high-level vector storage engine built on [seqdb](../seqdb) that provides persistent vector-like data structures. It supports multiple storage formats and computation strategies for different performance and space requirements.

## Key Features

- **Multiple storage variants**: Raw, compressed, lazy, eager, and computed vectors
- **Advanced compression**: Uses Pcodec for numerical data compression
- **Type safety**: Generic storage with zero-copy access
- **Versioning system**: Change tracking and rollback support
- **Hole management**: Efficient sparse data handling
- **Thread-safe**: Concurrent read operations

## Storage Variants

### RawVec - Uncompressed Storage
Fast, direct storage without compression.

```rust
use vecdb::{Database, RawVec, Version};
use std::path::Path;

let db = Database::open(Path::new("data"))?;
let mut vec: RawVec<usize, u32> = RawVec::forced_import(&db, "numbers", Version::TWO)?;

// Basic operations
vec.push(42);
vec.push(84);

// Reading with zero-copy when possible
let reader = vec.create_static_reader();
let value = vec.get_or_read(0, &reader)?.unwrap();
assert_eq!(*value, 42);
drop(reader);

// Updates and deletions
vec.update(0, 100)?;
let removed = vec.take(1, &vec.create_static_reader())?; // Creates a hole

vec.flush()?;
```

### CompressedVec - Space-Efficient Storage
Automatic compression using Pcodec for numerical data.

```rust
use vecdb::{Database, CompressedVec, Version};

let db = Database::open(Path::new("data"))?;
let mut vec: CompressedVec<usize, f64> =
    CompressedVec::forced_import(&db, "measurements", Version::TWO)?;

// Same API as RawVec but with automatic compression
for i in 0..1000 {
    vec.push(i as f64 * 3.14159);
}

vec.flush()?; // Data compressed on flush

// Reading transparently decompresses
let reader = vec.create_static_reader();
let value = vec.get_or_read(500, &reader)?;
```

### ComputedVec - Derived Data
On-demand or pre-computed vectors derived from other vectors.

```rust
use vecdb::{ComputedVecFrom1, Computation, Format};

// Source data
let mut source: RawVec<usize, f64> = RawVec::forced_import(&db, "source", Version::TWO)?;
source.push(2.0);
source.push(3.0);
source.flush()?;

// Computed vector (squares the source values)
let computed = ComputedVecFrom1::forced_import_or_init_from_1(
    &db,
    "squares",
    Version::TWO,
    Computation::Eager,
    Format::Compressed,
    source.boxed_iter(),
    |_index, iter| {
        iter.get(_index).map(|(_, value)| value.as_ref() * value.as_ref())
    },
)?;
```

## Core Operations

### Basic Vector Operations

```rust
let mut vec: RawVec<usize, i32> = RawVec::forced_import(&db, "data", Version::TWO)?;

// Adding elements
vec.push(10);
vec.push(20);
vec.push(30);

// Reading elements
let reader = vec.create_static_reader();
let value = vec.get_or_read(1, &reader)?;
drop(reader);

// Updating elements
vec.update(0, 15)?;

// Removing elements (creates holes)
let removed = vec.take(1, &vec.create_static_reader())?;

// Fill holes when adding new data
let new_index = vec.fill_first_hole_or_push(25)?;

// Persistence
vec.flush()?;
```

### Collection and Iteration

```rust
// Collect all values (skipping holes)
let values: Vec<i32> = vec.collect()?;

// Collect including holes as Option<T>
let with_holes: Vec<Option<i32>> = vec.collect_holed()?;

// Iterator support
for (index, value) in &vec {
    println!("vec[{}] = {}", index, value.as_ref());
}

// Range iteration
let last_5: Vec<i32> = vec.collect_signed_range(Some(-5), None)?;
```

### Version Control

```rust
use vecdb::Stamp;

// Save with version stamp
vec.stamped_flush(Stamp::new(42))?;

// Rollback to previous version
vec.rollback_stamp(Stamp::new(41))?;
```

## Type Requirements

### Index Types
Must implement `StoredIndex` trait (built-in for `usize`, `u32`, `u64`, etc.).

### Value Types

**For RawVec (`StoredRaw`)**:
- `FromBytes` + `IntoBytes` (zerocopy traits)
- `Clone` + `Copy` + `Debug` + `Send` + `Sync`

**For CompressedVec (`StoredCompressed`)**:
- All `StoredRaw` requirements
- Numerical types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`
- Custom types via `#[derive(StoredCompressed)]` with [vecdb_derive](../vecdb_derive)

## Performance Characteristics

| Operation | RawVec | CompressedVec | ComputedVec |
|-----------|--------|---------------|-------------|
| Random Read | O(1) | O(page_size) | O(computation) |
| Sequential Read | Fastest | Fast | Variable |
| Write | Fastest | Fast | N/A |
| Space Usage | 1.0x | 0.1x - 0.5x | Variable |

## Best Practices

1. **Choose the right variant**: RawVec for speed, CompressedVec for space, ComputedVec for derived data
2. **Manage readers**: Create readers for read operations, drop before mutations
3. **Batch operations**: Flush once after multiple operations for better performance
4. **Handle concurrent access**: Multiple readers are safe, coordinate writers externally

## Use Cases

- **Time-series data**: Compressed storage of sensor readings
- **Analytics**: Derived computations from base datasets
- **Caching**: Persistent memoization of expensive computations
- **Scientific computing**: Large numerical datasets with compression

## Error Handling

```rust
use vecdb::{Error, Result};

match vec.get_or_read(index, &reader) {
    Ok(Some(value)) => println!("Found: {:?}", value),
    Ok(None) => println!("Hole at index {}", index),
    Err(Error::IndexTooHigh) => println!("Index out of bounds"),
    Err(Error::DifferentVersion { .. }) => println!("Version mismatch"),
    Err(e) => println!("Other error: {}", e),
}
```

## Integration

Built on [seqdb](../seqdb) for:
- Page-aligned storage
- Cross-platform file locking
- Dynamic space management

---

*This README was generated by Claude Code*
