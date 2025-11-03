# rawdb

Single-file database with named regions and automatic space management.

## Install

```bash
cargo add rawdb
```

## What it is

A mmap-backed storage engine that manages multiple named regions in one file. Think filesystem-like abstraction where regions are files, but everything lives in a single physical file with automatic compaction.

## Why use it

**One file, many regions**: Create unlimited named regions in a single database file. No directory management, no file handle limits.

**Automatic space reclamation**: Removed regions become holes that are automatically reused. Hole punching reduces disk usage without manual compaction.

**Zero-copy access**: Direct mmap access. Read without copying into buffers.

**Thread-safe**: Clone the database handle and share across threads. Concurrent reads and writes are safe.

**Page-aligned**: All allocations are 4KB page-aligned. Regions grow automatically on demand.

## When to use it

- Multiple logical partitions without filesystem overhead
- Building higher-level data structures (vectors, arrays, indices)
- Need mmap performance with simpler management than raw files
- Append-mostly workloads with occasional region removal

## What it's not

Not a general-purpose database. No transactions, no queries, no schemas. Just regions of bytes with automatic layout management.

## Examples

See [examples/](examples/) for usage.
