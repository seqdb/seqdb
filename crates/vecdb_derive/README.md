# [vecdb_derive]

Procedural macros for the VecDB storage engine, providing automatic trait implementations for compressed vector types.

## Overview

This crate provides derive macros that automatically implement the necessary traits for storing and compressing custom data types in VecDB. The main macro `StoredCompressed` enables transparent compression for wrapper types.

## Derive Macros

### `#[derive(StoredCompressed)]`

Automatically implements compression traits for single-field tuple structs, enabling them to be stored efficiently in compressed vectors.

**Requirements:**
- Must be used on a single-field tuple struct
- The inner type must already implement `StoredCompressed`

**Example:**

```rust
use vecdb_derive::StoredCompressed;
use vecdb::CompressedVec;

// Define a wrapper type
#[derive(StoredCompressed)]
struct UserId(u32);

// Now UserId can be used in compressed vectors
type UserVec = CompressedVec<usize, UserId>;
```

**With Generics:**

```rust
#[derive(StoredCompressed)]
struct Wrapper<T>(T);

// The derive macro handles generic parameters automatically
// T must implement StoredCompressed for this to work
```

## Implementation Details

The macro generates implementations for:
- `StoredCompressed` trait with the appropriate `NumberType`
- `TransparentStoredCompressed` trait for zero-cost wrapper semantics

The generated code ensures that wrapper types have the same compression characteristics as their inner types, making them truly transparent for storage purposes.

## Usage Notes

- Only works with tuple structs containing exactly one field
- The inner type must implement the `StoredCompressed` trait
- Supports generic parameters with appropriate where clauses
- Designed for zero-overhead abstractions over compressed numeric types
