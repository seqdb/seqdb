# [vecdb]

A KISS (Keep It Simple, Stupid) index-value storage engine optimized for columnar data with transparent compression support.

## Overview

VecDB is an embedded database engine designed for high-performance columnar storage. It provides vector-like data structures that can be persisted to disk with optional compression, making it ideal for analytical workloads and time-series data.

## Key Features

- **Columnar storage**: Optimized for analytical queries and data compression
- **Embedded**: No separate server process - runs directly in your application
- **Index-free**: Uses array indices as keys, eliminating key storage overhead
- **Value-focused**: Only actual values are stored, maximizing space efficiency
- **Dual storage modes**: Choose between raw (fast access) or compressed (space efficient) storage
- **Transactional**: ACID-compliant operations with proper isolation
- **Multi-reader/writer**: Concurrent access support with fine-grained locking
- **Performance-optimized**: Non-portable design choices for maximum speed on supported platforms
- **Unix-focused**: Primarily designed for Unix-like systems

## Storage Variants

VecDB supports multiple vector implementations for different use cases:

### Raw Vectors (`RawVec`)
- Direct, uncompressed storage for maximum read/write speed
- Ideal for frequently accessed data and real-time applications

### Compressed Vectors (`CompressedVec`)
- Advanced compression using `pco` (Pcodec) for numerical data
- Significant space savings with acceptable performance trade-offs
- Perfect for analytical workloads and archival data

### Computed Vectors
- On-the-fly computation from other vectors
- Lazy evaluation for derived data sets
- Support for 1-3 input vector computations

### Eager/Lazy Variants
- Different loading and caching strategies
- Optimized for various memory and performance constraints

## Example Usage

### Raw Storage

```rust
use std::{path::Path, sync::Arc};
use vecdb::{RawVec, Database, Version};

let database = Database::open(Path::new("data"))?;
let mut vec: RawVec<usize, u32> = RawVec::forced_import(&database, "my_vec", Version::TWO)?;

// Push values
vec.push(42);
vec.push(84);

// Read values
let reader = vec.create_reader();
let value = vec.get_or_read(0, &reader)?; // Returns Result<Option<Cow<u32>>>

// Persist to disk
vec.flush()?;
```

### Compressed Storage

```rust
use vecdb::{CompressedVec, Database, Version};

let database = Database::open(Path::new("data"))?;
let mut vec: CompressedVec<usize, u32> = CompressedVec::forced_import(&database, "compressed_vec", Version::TWO)?;

// Same API as raw vectors, but with compression
vec.push(1000);
vec.flush()?;
```

## Architecture

VecDB is built on top of SeqDB for low-level storage management and provides:

- **Type-safe interfaces**: Generic vector types with compile-time type checking
- **Versioning system**: Schema evolution and backward compatibility
- **Stamping mechanism**: Track data freshness and updates
- **Hole management**: Efficient handling of deleted elements
- **Iterator support**: Standard Rust iterator patterns for data access

## Use Cases

- Time-series databases
- Analytical data processing
- Scientific computing datasets
- Financial market data
- IoT sensor data storage
- Any scenario requiring fast columnar access patterns

VecDB excels when you need the performance of in-memory data structures with the durability of persistent storage.
