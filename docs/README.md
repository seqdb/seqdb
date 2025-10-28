# [seqdb]

A high-performance, embedded database system designed for analytical workloads and columnar data storage.

## Overview

SeqDB is a complete database solution built around two core principles: **simplicity** and **performance**. The system provides both low-level storage primitives and high-level vector abstractions, making it suitable for everything from time-series databases to scientific computing applications.

## Architecture

The project consists of three complementary crates that work together to provide a complete storage solution:

```
VecDB (High-level vectors)
├─ Raw/Compressed vectors
├─ Type-safe interfaces
└─ Analytical operations
         │
         ▼
SeqDB (Low-level storage)
├─ Region management
└─ Space optimization
```

### Core Crates

- **[`seqdb`](crates/seqdb/)** - The foundational storage engine providing file management, dynamic regions, and efficient space utilization
- **[`vecdb`](crates/vecdb/)** - High-level columnar storage with vector abstractions, compression support, and analytical operations
- **[`vecdb_derive`](crates/vecdb_derive/)** - Procedural macros for automatic trait implementations and transparent compression support

## Key Features

### Storage Engine (SeqDB)

- **Dynamic region allocation** with automatic growth and compaction
- **Hole punching** to reclaim disk space efficiently
- **Page-aligned operations** optimized for filesystem performance
- **Thread-safe** concurrent access with fine-grained locking

### Vector Database (VecDB)

- **Columnar storage** optimized for analytical queries
- **Multiple storage formats**: raw (speed) vs compressed (space)
- **Advanced compression** using pco (Pcodec) for numerical data
- **Type-safe generics** with compile-time guarantees
- **Iterator patterns** following Rust conventions
- **Versioning system** for schema evolution

## Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
vecdb = "0.0.1"
```

### Basic Usage

```rust
use std::{path::Path, sync::Arc};
use vecdb::{RawVec, Database, Version};

// Create database
let db = Database::open(Path::new("my_data"))?;

// Create a vector for storing numbers
let mut vec: RawVec<usize, u32> = RawVec::forced_import(&db, "numbers", Version::TWO)?;

// Add data
vec.push(100);
vec.push(200);
vec.push(300);

// Read data
for (index, value) in vec.into_iter() {
    println!("Index {}: {}", index, value);
}

// Persist to disk
vec.flush()?;
```

### Compressed Storage

```rust
use vecdb::{CompressedVec, Database, Version};

let db = Database::open(Path::new("my_data"))?;

let mut compressed: CompressedVec<usize, f64> =
    CompressedVec::forced_import(&db, "sensor_data", Version::TWO)?;

// Same API, automatic compression
compressed.push(23.5);
compressed.push(24.1);
compressed.flush()?;
```

## Performance Characteristics

- **Raw vectors**: Optimized for latency-critical applications
- **Compressed vectors**: 2-10x space savings with modest CPU overhead
- **Concurrent access**: Multiple readers and writers with minimal contention
- **Space efficiency**: Automatic hole punching and region compaction

## Use Cases

### Time-Series Databases

Store sensor data, financial ticks, or metrics with efficient compression and fast range queries.

### Analytical Workloads

Process large datasets with columnar access patterns and transparent compression.

### Scientific Computing

Handle numerical datasets with type safety and performance optimizations.

### Embedded Systems

Lightweight, dependency-minimal database for resource-constrained environments.

## Platform Support

- **Primary**: Unix-like systems (Linux, macOS, BSD)
- **Architecture**: x86_64, ARM64
- **Minimum Rust**: 1.89+ (edition 2024)

## Development

```bash
# Run all tests
cargo test --workspace

# Run examples (from specific crates)
cargo run --example db -p seqdb
cargo run --example raw -p vecdb
cargo run --example compressed -p vecdb

# Build with optimizations
cargo build --release
```

## License

MIT Licensed - see [LICENSE.md](LICENSE.md) for details.

## Contributing

This project prioritizes simplicity and performance. Contributions should maintain these principles while adding meaningful functionality.
