# rawdb

Single-file low-level space efficient storage engine with filesystem-like API.

It features:

- Multiple named regions in one file
- Automatic space reclamation via hole punching
- Regions grow and move automatically as needed
- Zero-copy mmap access
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
    let db = Database::open("my_db")?;

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
    db.punch_holes()?;

    Ok(())
}
```

## Examples

See [examples/](examples/) for usage.
