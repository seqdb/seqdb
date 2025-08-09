# [seqdb]

A high-performance, memory-mapped database engine for sequential data storage with dynamic region management.

## Overview

SeqDB provides a sophisticated storage system built around memory-mapped files and dynamic region allocation. It efficiently handles variable-sized data regions with features like hole punching, region defragmentation, and automatic file growth.

## Key Features

- **Memory-mapped I/O**: Direct memory access to disk-backed data for optimal performance
- **Dynamic regions**: Create and manage named data regions that can grow and shrink as needed
- **Space reclamation**: Automatic hole punching and region compaction to minimize disk usage
- **Thread-safe**: Built with `parking_lot` for efficient concurrent access
- **Cross-platform**: Support for Linux, macOS, and other Unix-like systems

## Core Concepts

- **Regions**: Named, variable-size data containers within the database file
- **Layout**: Tracks region positions and manages free space efficiently
- **Page-aligned**: All operations work with 4KB page boundaries for optimal filesystem performance
- **Reserved space**: Regions pre-allocate space to reduce fragmentation during growth

## Example Usage

```rust
use std::path::Path;
use seqdb::{SeqDB, PAGE_SIZE};

// Open or create a database
let db = SeqDB::open(Path::new("my_database"))?;

// Create a new region
let (region_id, _) = db.create_region_if_needed("my_region")?;

// Write data to the region
db.write_all_to_region(region_id.into(), b"Hello, world!")?;

// Read data back
let reader = db.create_region_reader(region_id.into())?;
// ... read operations

// Flush changes to disk
db.flush()?;
```

## Architecture

The database consists of three main components:
- A data file containing the actual region content
- Region metadata tracking each region's location, size, and ID mapping
- Layout information managing free space and region placement

SeqDB handles complex scenarios like region growth, movement, and space optimization automatically while maintaining data consistency.
