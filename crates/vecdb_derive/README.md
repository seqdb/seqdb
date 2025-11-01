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
