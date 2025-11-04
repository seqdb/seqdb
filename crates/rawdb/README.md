# rawdb

Single-file, low-level and space efficient storage engine with filesystem-like API.

It features:

- Multiple named regions in one file
- Automatic space reclamation via hole punching
- Regions grow and move automatically as needed
- Optional zero-copy mmap access
- Thread-safe with concurrent reads and writes
- Page-aligned allocations (4KB)
- Persistence only on flush
- Foundation for higher-level abstractions (e.g., [`vecdb`](../vecdb/README.md))

It is not:

- A general-purpose database (no transactions, queries, or schemas)

## Install

```bash
cargo add rawdb
```

## Usage

```rust
use rawdb::{Database, Result};

fn main() -> Result<()> {
    // open database
    let temp_dir = tempfile::TempDir::new()?;
    let db = Database::open(temp_dir.path())?;

    // create regions
    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;

    // write data
    db.write_all_to_region(&region1, &[0, 1, 2, 3, 4])?;
    db.write_all_to_region(&region2, &[5, 6, 7, 8, 9])?;

    // read via mmap
    let data = &db.mmap()[0..5];

    // flush to disk
    db.flush()?;

    // remove region (space becomes reusable hole)
    db.remove_region(region1)?;

    db.flush()?; // Should be `db.flush_then_punch()?` but doesn't work with `TempDir`

    Ok(())
}
```

## Durability

Operations are durable after calling `flush()`. Before flush, writes are visible in memory but not guaranteed to survive crashes.

**Design:**
- **4KB metadata entries**: Atomic writes per region. IDs embedded in metadata.
- **Single metadata file**: Rebuilt into HashMap on startup for O(1) lookups.
- **No WAL**: Simple design. Metadata is always consistent after flush.

**Region writes:**
- Expand in-place when possible (last region or adjacent hole)
- Copy-on-write to new location when expansion needed
- Metadata written immediately but only durable after `flush()`

**Recovery:**
On open, reads all metadata entries and rebuilds in-memory structures. Empty IDs indicate deleted regions.

## Examples

See [examples/](examples/) for usage.
