# rawdb

Non-transactional embedded storage engine with a filesystem-like API.

It features:

- Multiple named regions in one file
- Automatic space reclamation via hole punching
- Regions grow and move automatically as needed
- Zero-copy mmap access
- Thread-safe with concurrent reads and writes
- Page-aligned allocations (4KB)
- Crash-consistent with explicit flush
- Foundation for higher-level abstractions (e.g., [`vecdb`](../vecdb/README.md))

It is not:

- A transactional database (no ACID, transactions, or rollback)
- A query engine (no SQL, indexes, or schemas)

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

    db.flush()?; // Should be `db.compact()?` but doesn't work with with doc-tests

    Ok(())
}
```

## Durability

Operations become durable after calling `flush()`. Before flush, writes are visible in memory but not guaranteed to survive crashes.

**Design:**
- **4KB metadata entries**: Atomic page-sized writes per region with embedded IDs
- **Single metadata file**: Rebuilt into HashMap on startup for O(1) lookups
- **No WAL**: Simple design with proper write ordering for consistency
- **Dirty tracking**: Metadata changes tracked in-memory, batch-written on flush

**Write ordering:**
1. Data writes update mmap (in-memory)
2. Metadata changes tracked with dirty flag (in-memory)
3. Holes from moves/removes marked as pending (not reusable until flush)
4. `flush()` syncs mmap first, then writes dirty metadata, then syncs metadata file, then promotes pending holes
5. Ensures metadata never points to unflushed data and old locations aren't reused prematurely (crash-consistent COW)

**Region operations:**
- Expand in-place when possible (last region or adjacent hole)
- Copy-on-write to new location when expansion needed
- All changes stay in memory until `flush()` makes them durable

**Recovery:**
On open, reads all metadata entries and rebuilds in-memory structures. Deleted regions are identified by zeroed metadata.
