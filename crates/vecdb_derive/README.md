# vecdb_derive

Derive macros for [`vecdb`](../vecdb/) compression support.

Automatically implements compression traits for custom wrapper types, enabling them to work with `CompressedVec`.

## Install

```bash
cargo add vecdb --features derive
```

## Usage

```rust
use vecdb_derive::Compressable;

#[derive(Compressable)]
struct Timestamp(u64);

// Now works with CompressedVec
let mut vec: CompressedVec<usize, Timestamp> = ...;
vec.push(Timestamp(12345));
```

## `#[derive(Compressable)]`

Implements `Compressable` for single-field tuple structs. The wrapper inherits compression characteristics from the inner type.

**Requirements:**
- Must be a tuple struct with exactly one field
- Inner type must implement `Compressable`
