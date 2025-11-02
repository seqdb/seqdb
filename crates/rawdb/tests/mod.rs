use rawdb::{Database, Result, PAGE_SIZE};
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// Helper to create a temporary test database
fn setup_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

#[test]
fn test_database_creation() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Database should start empty
    assert_eq!(db.regions().index_to_region().len(), 0);
    assert_eq!(db.layout().start_to_region().len(), 0);
    assert_eq!(db.layout().start_to_hole().len(), 0);

    Ok(())
}

#[test]
fn test_create_single_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test_region")?;

    // Verify region properties
    let meta = region.meta().read();
    assert_eq!(meta.start(), 0);
    assert_eq!(meta.len(), 0);
    assert_eq!(meta.reserved(), PAGE_SIZE);
    drop(meta);

    // Verify it's tracked in regions
    let regions = db.regions();
    assert_eq!(regions.index_to_region().len(), 1);
    assert!(regions.get_region_from_id("test_region").is_some());

    // Verify it's tracked in layout
    let layout = db.layout();
    assert_eq!(layout.start_to_region().len(), 1);
    assert!(layout.start_to_hole().is_empty());

    Ok(())
}

#[test]
fn test_create_region_idempotent() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("test")?;
    let region2 = db.create_region_if_needed("test")?;

    // Should return same region
    assert_eq!(region1.index(), region2.index());
    assert_eq!(db.regions().index_to_region().len(), 1);

    Ok(())
}

#[test]
fn test_write_to_region_within_reserved() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    let data = b"Hello, World!";

    db.write_all_to_region(&region, data)?;

    // Verify data was written
    let meta = region.meta().read();
    assert_eq!(meta.len(), data.len() as u64);
    assert_eq!(meta.reserved(), PAGE_SIZE);
    let start = meta.start();
    drop(meta);

    let mmap = db.mmap();
    assert_eq!(
        &mmap[start as usize..(start + data.len() as u64) as usize],
        data
    );

    Ok(())
}

#[test]
fn test_write_append() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    db.write_all_to_region(&region, b"Hello")?;
    db.write_all_to_region(&region, b", World!")?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), 13);
    let start = meta.start();
    drop(meta);

    let mmap = db.mmap();
    assert_eq!(
        &mmap[start as usize..(start + 13) as usize],
        b"Hello, World!"
    );

    Ok(())
}

#[test]
fn test_write_at_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    db.write_all_to_region(&region, b"Hello, World!")?;
    db.write_all_to_region_at(&region, b"Rust!", 7)?;

    let meta = region.meta().read();
    let start = meta.start();
    drop(meta);

    let mmap = db.mmap();
    assert_eq!(
        &mmap[start as usize..(start + 13) as usize],
        b"Hello, Rust!!"
    );

    Ok(())
}

#[test]
fn test_write_exceeds_reserved() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Write more than PAGE_SIZE to trigger expansion
    let large_data = vec![1u8; (PAGE_SIZE + 100) as usize];
    db.write_all_to_region(&region, &large_data)?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), large_data.len() as u64);
    assert!(meta.reserved() >= PAGE_SIZE * 2);

    Ok(())
}

#[test]
fn test_truncate_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    db.write_all_to_region(&region, b"Hello, World!")?;

    let meta_before = region.meta().read();
    assert_eq!(meta_before.len(), 13);
    drop(meta_before);

    db.truncate_region(&region, 5)?;

    let meta_after = region.meta().read();
    assert_eq!(meta_after.len(), 5);

    Ok(())
}

#[test]
fn test_truncate_errors() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    db.write_all_to_region(&region, b"Hello")?;

    // Truncating beyond length should error
    let result = db.truncate_region(&region, 10);
    assert!(result.is_err());

    // Truncating to same length should be OK
    let result = db.truncate_region(&region, 5);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_remove_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    let index = region.index();

    db.write_all_to_region(&region, b"Hello")?;

    // Remove region
    let removed = db.remove_region(region)?;
    assert!(removed.is_some());

    // Verify removal
    let regions = db.regions();
    assert!(regions.get_region_from_id("test").is_none());
    assert!(regions.get_region_from_index(index).is_none());

    // Layout should have a hole now
    let layout = db.layout();
    assert_eq!(layout.start_to_region().len(), 0);
    assert_eq!(layout.start_to_hole().len(), 1);

    Ok(())
}

#[test]
fn test_multiple_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let region3 = db.create_region_if_needed("region3")?;

    // Write different data to each
    db.write_all_to_region(&region1, b"First")?;
    db.write_all_to_region(&region2, b"Second")?;
    db.write_all_to_region(&region3, b"Third")?;

    // Verify all exist
    assert_eq!(db.regions().index_to_region().len(), 3);
    assert_eq!(db.layout().start_to_region().len(), 3);

    // Verify data integrity
    let mmap = db.mmap();

    let meta1 = region1.meta().read();
    assert_eq!(
        &mmap[meta1.start() as usize..(meta1.start() + 5) as usize],
        b"First"
    );
    drop(meta1);

    let meta2 = region2.meta().read();
    assert_eq!(
        &mmap[meta2.start() as usize..(meta2.start() + 6) as usize],
        b"Second"
    );
    drop(meta2);

    let meta3 = region3.meta().read();
    assert_eq!(
        &mmap[meta3.start() as usize..(meta3.start() + 5) as usize],
        b"Third"
    );

    Ok(())
}

#[test]
fn test_region_reuse_after_removal() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let _region2 = db.create_region_if_needed("region2")?;
    let index1 = region1.index();

    // Remove first region
    db.remove_region(region1)?;

    // Create a new region - should reuse the slot
    let region3 = db.create_region_if_needed("region3")?;
    assert_eq!(region3.index(), index1);

    // Verify only 2 regions exist
    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 2);
    assert!(regions.get_region_from_id("region1").is_none());
    assert!(regions.get_region_from_id("region2").is_some());
    assert!(regions.get_region_from_id("region3").is_some());

    Ok(())
}

#[test]
fn test_hole_filling() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let _region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let _region3 = db.create_region_if_needed("region3")?;

    // Remove middle region to create a hole
    db.remove_region(region2)?;

    let layout = db.layout();
    assert_eq!(layout.start_to_hole().len(), 1);
    drop(layout);

    // Create new region - should fill the hole
    let _region4 = db.create_region_if_needed("region4")?;

    let layout = db.layout();
    // Hole should be gone since new region takes PAGE_SIZE which fills it exactly
    assert_eq!(layout.start_to_hole().len(), 0);

    Ok(())
}

#[test]
fn test_persistence() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();
    dbg!(&path);

    // Create and populate database
    {
        let db = Database::open(path)?;
        let region = db.create_region_if_needed("persistent")?;
        db.write_all_to_region(&region, b"Persisted data")?;
        db.flush()?;
    }

    // Reopen and verify
    {
        let db = Database::open(path)?;
        let regions = db.regions();
        let region = regions
            .get_region_from_id("persistent")
            .expect("Region should persist");

        let meta = region.meta().read();
        assert_eq!(meta.len(), 14);
        let start = meta.start();
        drop(meta);

        let mmap = db.mmap();
        assert_eq!(
            &mmap[start as usize..(start + 14) as usize],
            b"Persisted data"
        );
    }

    Ok(())
}

#[test]
fn test_reader() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    db.write_all_to_region(&region, b"Hello, World!")?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"Hello, World!");
    assert_eq!(reader.read(0, 5), b"Hello");
    assert_eq!(reader.read(7, 5), b"World");

    Ok(())
}

#[test]
fn test_retain_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    db.create_region_if_needed("keep1")?;
    db.create_region_if_needed("remove1")?;
    db.create_region_if_needed("keep2")?;
    db.create_region_if_needed("remove2")?;

    let mut keep_set = std::collections::HashSet::new();
    keep_set.insert("keep1".to_string());
    keep_set.insert("keep2".to_string());

    db.retain_regions(keep_set)?;

    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 2);
    assert!(regions.get_region_from_id("keep1").is_some());
    assert!(regions.get_region_from_id("keep2").is_some());
    assert!(regions.get_region_from_id("remove1").is_none());
    assert!(regions.get_region_from_id("remove2").is_none());

    Ok(())
}

#[test]
fn test_region_defragmentation() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;

    dbg!(0);

    // Write small data first
    db.write_all_to_region(&region1, b"small")?;

    dbg!(1);
    // Write large data to region1 - should move it to end
    let large_data = vec![1u8; (PAGE_SIZE * 2) as usize];
    db.write_all_to_region(&region1, &large_data)?;

    dbg!(2);
    // region1 should have moved, leaving a hole
    let layout = db.layout();
    assert!(layout.start_to_hole().len() == 1);

    dbg!(3);
    // region2 should still be at its original position
    let meta2 = region2.meta().read();
    assert_eq!(meta2.start(), PAGE_SIZE);
    dbg!(4);

    Ok(())
}

#[test]
fn test_concurrent_region_creation() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Arc::new(Database::open(temp.path())?);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let region_name = format!("region_{}", i);
                db.create_region_if_needed(&region_name)
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify all regions created
    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 10);

    Ok(())
}

#[test]
fn test_set_min_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    db.set_min_regions(100)?;

    // File should be large enough for 100 regions
    let file_len = db.file_len()?;
    assert!(file_len >= 100 * PAGE_SIZE);

    Ok(())
}

#[test]
fn test_large_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("large")?;

    // Write 1MB of data
    let large_data = vec![42u8; 1024 * 1024];
    db.write_all_to_region(&region, &large_data)?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), large_data.len() as u64);
    let start = meta.start();
    drop(meta);

    // Verify data
    let mmap = db.mmap();
    assert_eq!(
        &mmap[start as usize..(start + large_data.len() as u64) as usize],
        &large_data[..]
    );

    Ok(())
}

#[test]
fn test_truncate_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    db.write_all_to_region(&region, b"Hello, World!")?;

    let meta_before = region.meta().read();
    assert_eq!(meta_before.len(), 13);
    drop(meta_before);

    // Truncate write - should set length to exactly the written data
    db.truncate_write_all_to_region(&region, 7, b"Rust")?;

    let meta_after = region.meta().read();
    assert_eq!(meta_after.len(), 11); // 7 + 4
    let start = meta_after.start();
    drop(meta_after);

    let mmap = db.mmap();
    assert_eq!(&mmap[start as usize..(start + 11) as usize], b"Hello, Rust");

    Ok(())
}

#[test]
fn test_punch_holes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Write large data then truncate
    let large_data = vec![1u8; (PAGE_SIZE * 2) as usize];
    db.write_all_to_region(&region, &large_data)?;
    db.truncate_region(&region, 100)?;

    // Flush and punch holes
    db.flush_then_punch()?;

    // Should still be able to read the data
    let meta = region.meta().read();
    assert_eq!(meta.len(), 100);

    Ok(())
}

#[test]
fn test_write_at_invalid_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    db.write_all_to_region(&region, b"Hello")?;

    // Writing beyond length should fail
    let result = db.write_all_to_region_at(&region, b"World", 10);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_empty_region_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("empty")?;

    // Reading empty region
    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"");
    drop(reader);

    // Truncating empty region to 0 should work
    db.truncate_region(&region, 0)?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), 0);

    Ok(())
}

#[test]
fn test_region_metadata_updates() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Initial state
    {
        let meta = region.meta().read();
        assert_eq!(meta.start(), 0);
        assert_eq!(meta.len(), 0);
        assert_eq!(meta.reserved(), PAGE_SIZE);
    }

    // After first write
    db.write_all_to_region(&region, b"Hello")?;
    {
        let meta = region.meta().read();
        assert_eq!(meta.len(), 5);
        assert_eq!(meta.reserved(), PAGE_SIZE);
    }

    // After expansion
    let large = vec![1u8; (PAGE_SIZE * 3) as usize];
    db.write_all_to_region(&region, &large)?;
    {
        let meta = region.meta().read();
        assert_eq!(meta.len(), 5 + large.len() as u64);
        assert!(meta.reserved() >= PAGE_SIZE * 4);
    }

    Ok(())
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn test_complex_region_lifecycle() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create multiple regions
    let r1 = db.create_region_if_needed("region1")?;
    let r2 = db.create_region_if_needed("region2")?;
    let r3 = db.create_region_if_needed("region3")?;

    // Write to all regions
    db.write_all_to_region(&r1, b"Data for region 1")?;
    db.write_all_to_region(&r2, b"Data for region 2")?;
    db.write_all_to_region(&r3, b"Data for region 3")?;

    // Remove middle region
    db.remove_region(r2)?;

    // Verify hole exists
    {
        let layout = db.layout();
        assert_eq!(layout.start_to_hole().len(), 1);
    }

    // Create new region that should reuse the hole
    let r4 = db.create_region_if_needed("region4")?;
    db.write_all_to_region(&r4, b"Fills the hole")?;

    // Verify hole was filled
    {
        let layout = db.layout();
        assert_eq!(layout.start_to_hole().len(), 0);
    }

    // Write large data to trigger region movement (overwrite from start)
    let large = vec![42u8; (PAGE_SIZE * 3) as usize];
    db.write_all_to_region_at(&r4, &large, 0)?;

    // Verify r4 moved and created a hole
    {
        let layout = db.layout();
        assert!(!layout.start_to_hole().is_empty());
    }

    // Verify all data is still correct
    {
        let reader = r1.create_reader();
        assert_eq!(reader.read_all(), b"Data for region 1");
        drop(reader);
    }

    {
        let reader = r3.create_reader();
        assert_eq!(reader.read_all(), b"Data for region 3");
        drop(reader);
    }

    {
        let reader = r4.create_reader();
        assert_eq!(reader.read_all(), &large[..]);
        drop(reader);
    }

    Ok(())
}

#[test]
fn test_many_small_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create 50 small regions
    let mut regions = Vec::new();
    for i in 0..50 {
        let name = format!("region_{}", i);
        let region = db.create_region_if_needed(&name)?;
        let data = format!("Data for region {}", i);
        db.write_all_to_region(&region, data.as_bytes())?;
        regions.push(Some(region));
    }

    // Verify all regions
    for (i, region_opt) in regions.iter().enumerate() {
        let region = region_opt.as_ref().unwrap();
        let reader = region.create_reader();
        let expected = format!("Data for region {}", i);
        assert_eq!(reader.read_all(), expected.as_bytes());
        drop(reader);
    }

    // Remove every other region
    for i in (0..50).step_by(2) {
        let region = regions[i].take().unwrap();
        db.remove_region(region)?;
    }

    // Verify remaining regions
    for i in (1..50).step_by(2) {
        let region = regions[i].as_ref().unwrap();
        let reader = region.create_reader();
        let expected = format!("Data for region {}", i);
        assert_eq!(reader.read_all(), expected.as_bytes());
        drop(reader);
    }

    Ok(())
}

#[test]
fn test_interleaved_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let r1 = db.create_region_if_needed("r1")?;
    let r2 = db.create_region_if_needed("r2")?;
    let r3 = db.create_region_if_needed("r3")?;

    // Interleave writes
    db.write_all_to_region(&r1, b"Start1")?;
    db.write_all_to_region(&r2, b"Start2")?;
    db.write_all_to_region(&r3, b"Start3")?;

    db.write_all_to_region(&r1, b" More1")?;
    db.write_all_to_region(&r2, b" More2")?;

    // Truncate one
    db.truncate_region(&r3, 3)?;

    // Continue writing
    db.write_all_to_region(&r1, b" End1")?;
    db.write_all_to_region_at(&r2, b"X", 0)?;

    // Verify results
    {
        let reader = r1.create_reader();
        assert_eq!(reader.read_all(), b"Start1 More1 End1");
        drop(reader);
    }

    {
        let reader = r2.create_reader();
        assert_eq!(reader.read_all(), b"Xtart2 More2");
        drop(reader);
    }

    {
        let meta = r3.meta().read();
        assert_eq!(meta.len(), 3);
    }

    Ok(())
}

#[test]
fn test_persistence_with_holes() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();

    // Create database with regions and holes
    {
        let db = Database::open(path)?;

        let r1 = db.create_region_if_needed("keep1")?;
        let r2 = db.create_region_if_needed("remove")?;
        let r3 = db.create_region_if_needed("keep2")?;

        db.write_all_to_region(&r1, b"Keep this 1")?;
        db.write_all_to_region(&r2, b"Remove this")?;
        db.write_all_to_region(&r3, b"Keep this 2")?;

        db.remove_region(r2)?;
        db.flush()?;
    }

    // Reopen and verify
    {
        let db = Database::open(path)?;

        let regions = db.regions();
        assert!(regions.get_region_from_id("keep1").is_some());
        assert!(regions.get_region_from_id("remove").is_none());
        assert!(regions.get_region_from_id("keep2").is_some());

        let r1 = regions.get_region_from_id("keep1").unwrap();
        let r3 = regions.get_region_from_id("keep2").unwrap();

        let reader1 = r1.create_reader();
        assert_eq!(reader1.read_all(), b"Keep this 1");
        drop(reader1);

        let reader3 = r3.create_reader();
        assert_eq!(reader3.read_all(), b"Keep this 2");
        drop(reader3);

        // Verify hole still exists
        let layout = db.layout();
        assert!(!layout.start_to_hole().is_empty());
    }

    Ok(())
}

#[test]
fn test_region_growth_patterns() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("growing")?;

    // Grow gradually
    for i in 0..10 {
        let data = vec![i as u8; 1000];
        db.write_all_to_region(&region, &data)?;
    }

    let meta = region.meta().read();
    assert_eq!(meta.len(), 10_000);

    // Verify all data
    let reader = region.create_reader();
    let all_data = reader.read_all();
    for i in 0..10 {
        let chunk = &all_data[i * 1000..(i + 1) * 1000];
        assert!(chunk.iter().all(|&b| b == i as u8));
    }

    Ok(())
}

#[test]
fn test_write_at_boundary_conditions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Write at start
    db.write_all_to_region(&region, b"0123456789")?;

    // Write at exact length boundary
    db.write_all_to_region_at(&region, b"ABC", 10)?;

    // Write at position 0
    db.write_all_to_region_at(&region, b"X", 0)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"X123456789ABC");

    Ok(())
}

#[test]
fn test_multiple_flushes() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();

    {
        let db = Database::open(path)?;
        let r = db.create_region_if_needed("test")?;

        db.write_all_to_region(&r, b"Version 1")?;
        db.flush()?;

        db.write_all_to_region(&r, b" Version 2")?;
        db.flush()?;

        db.write_all_to_region(&r, b" Version 3")?;
        db.flush()?;
    }

    {
        let db = Database::open(path)?;
        let regions = db.regions();
        let r = regions.get_region_from_id("test").unwrap();

        let reader = r.create_reader();
        assert_eq!(reader.read_all(), b"Version 1 Version 2 Version 3");
    }

    Ok(())
}

#[test]
fn test_hole_coalescing() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create 5 regions
    let mut regions: Vec<_> = (0..5)
        .map(|i| db.create_region_if_needed(&format!("r{}", i)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(Some)
        .collect();

    // Write small data to each
    for r in &regions {
        db.write_all_to_region(r.as_ref().unwrap(), b"data")?;
    }

    // Remove regions 1, 2, 3 to create adjacent holes
    db.remove_region(regions[1].take().unwrap())?;
    db.remove_region(regions[2].take().unwrap())?;
    db.remove_region(regions[3].take().unwrap())?;

    // Check that holes were coalesced
    let layout = db.layout();
    // Should have 1 large hole, not 3 separate ones
    let holes = layout.start_to_hole();
    assert_eq!(holes.len(), 1);

    // The single hole should span all 3 removed regions
    let hole_size = holes.values().next().unwrap();
    assert_eq!(*hole_size, PAGE_SIZE * 3);

    Ok(())
}

#[test]
fn test_stress_region_creation_and_removal() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create and remove regions in a cycle
    for cycle in 0..5 {
        // Create 20 regions
        let regions: Vec<_> = (0..20)
            .map(|i| {
                let name = format!("cycle_{}_region_{}", cycle, i);
                db.create_region_if_needed(&name)
            })
            .collect::<Result<Vec<_>>>()?;

        // Write to each
        for (i, r) in regions.iter().enumerate() {
            let data = format!("Cycle {} Region {}", cycle, i);
            db.write_all_to_region(r, data.as_bytes())?;
        }

        // Verify
        for (i, r) in regions.iter().enumerate() {
            let reader = r.create_reader();
            let expected = format!("Cycle {} Region {}", cycle, i);
            assert_eq!(reader.read_all(), expected.as_bytes());
            drop(reader);
        }

        // Remove all
        for r in regions {
            db.remove_region(r)?;
        }

        // Verify all gone
        let reg = db.regions();
        assert_eq!(reg.id_to_index().len(), 0);
    }

    Ok(())
}

#[test]
fn test_mixed_size_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("mixed")?;

    // Write various sizes
    db.write_all_to_region(&region, b"tiny")?;
    db.write_all_to_region(&region, &[1u8; 100])?;
    db.write_all_to_region(&region, &[2u8; 1000])?;
    db.write_all_to_region(&region, &[3u8; 10000])?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), 4 + 100 + 1000 + 10000);

    // Verify each section
    let reader = region.create_reader();
    assert_eq!(&reader.read(0, 4), b"tiny");
    assert!(reader.read(4, 100).iter().all(|&b| b == 1));
    assert!(reader.read(104, 1000).iter().all(|&b| b == 2));
    assert!(reader.read(1104, 10000).iter().all(|&b| b == 3));

    Ok(())
}

// ============================================================================
// Concurrent Operations Tests
// ============================================================================

#[test]
fn test_concurrent_writes_to_different_regions() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Arc::new(Database::open(temp.path())?);

    // Create regions upfront
    let regions: Vec<_> = (0..10)
        .map(|i| db.create_region_if_needed(&format!("region_{}", i)))
        .collect::<Result<Vec<_>>>()?;

    // Write to different regions concurrently
    let handles: Vec<_> = regions
        .into_iter()
        .enumerate()
        .map(|(i, region)| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let data = vec![i as u8; 1000];
                db.write_all_to_region(&region, &data)
            })
        })
        .collect();

    // Wait for all writes
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify all data
    for i in 0..10 {
        let regions = db.regions();
        let region = regions
            .get_region_from_id(&format!("region_{}", i))
            .unwrap();
        let reader = region.create_reader();
        let data = reader.read_all();
        assert_eq!(data.len(), 1000);
        assert!(data.iter().all(|&b| b == i as u8));
    }

    Ok(())
}

#[test]
fn test_concurrent_reads() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let db = Arc::new(db);

    let region = db.create_region_if_needed("shared")?;
    let data = b"Shared data for concurrent reads";
    db.write_all_to_region(&region, data)?;

    // Multiple threads reading simultaneously
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let regions = db.regions();
                let region = regions.get_region_from_id("shared").unwrap();
                let reader = region.create_reader();
                assert_eq!(reader.read_all(), b"Shared data for concurrent reads");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

// ============================================================================
// Reader Edge Cases
// ============================================================================

#[test]
fn test_reader_prefixed() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    db.write_all_to_region(&region, b"0123456789ABCDEF")?;

    let reader = region.create_reader();

    // Test prefixed reads
    let prefixed = reader.prefixed(5);
    assert!(prefixed.starts_with(b"56789ABCDEF"));

    let prefixed_at_start = reader.prefixed(0);
    assert!(prefixed_at_start.starts_with(b"0123456789ABCDEF"));

    Ok(())
}

#[test]
fn test_reader_unchecked_read() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    db.write_all_to_region(&region, b"Hello World")?;

    let reader = region.create_reader();

    // Unchecked read within bounds should work
    let data = reader.unchecked_read(0, 5);
    assert_eq!(data, b"Hello");

    let data = reader.unchecked_read(6, 5);
    assert_eq!(data, b"World");

    Ok(())
}

#[test]
#[should_panic]
fn test_reader_bounds_check() {
    let (db, _temp) = setup_test_db().unwrap();

    let region = db.create_region_if_needed("test").unwrap();
    db.write_all_to_region(&region, b"Short").unwrap();

    let reader = region.create_reader();

    // This should panic due to bounds check
    let _ = reader.read(0, 100);
}

// ============================================================================
// Extreme Cases
// ============================================================================

#[test]
fn test_very_long_region_names() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create regions with very long names
    let long_name = "a".repeat(1000);
    let region = db.create_region_if_needed(&long_name)?;
    db.write_all_to_region(&region, b"data")?;

    // Verify it persists
    db.flush()?;

    let regions = db.regions();
    let retrieved = regions.get_region_from_id(&long_name);
    assert!(retrieved.is_some());

    Ok(())
}

#[test]
fn test_zero_byte_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("empty_writes")?;

    // Write zero bytes
    db.write_all_to_region(&region, b"")?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), 0);
    drop(meta);

    // Write some data, then write zero bytes again
    db.write_all_to_region(&region, b"Hello")?;
    db.write_all_to_region(&region, b"")?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), 5);

    Ok(())
}

#[test]
fn test_alternating_write_and_truncate() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("oscillating")?;

    for cycle in 0..10 {
        // Grow
        let data = vec![cycle as u8; 1000];
        db.write_all_to_region(&region, &data)?;

        let meta = region.meta().read();
        let expected_len = if cycle == 0 { 1000 } else { 100 + 1000 };
        assert_eq!(meta.len(), expected_len);
        drop(meta);

        // Shrink
        db.truncate_region(&region, 100)?;

        let meta = region.meta().read();
        assert_eq!(meta.len(), 100);
        drop(meta);
    }

    Ok(())
}

// ============================================================================
// Persistence Edge Cases
// ============================================================================

#[test]
fn test_retain_regions_edge_cases() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create some regions
    db.create_region_if_needed("keep1")?;
    db.create_region_if_needed("keep2")?;
    db.create_region_if_needed("remove1")?;

    // Retain with empty set - should remove all
    let empty_set = std::collections::HashSet::new();
    db.retain_regions(empty_set)?;

    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 0);

    Ok(())
}

// ============================================================================
// Layout and Hole Management
// ============================================================================

#[test]
fn test_complex_fragmentation_scenario() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create pattern: region, region, region, region, region
    let r0 = db.create_region_if_needed("r0")?;
    let r1 = db.create_region_if_needed("r1")?;
    let r2 = db.create_region_if_needed("r2")?;
    let r3 = db.create_region_if_needed("r3")?;
    let r4 = db.create_region_if_needed("r4")?;

    for r in [&r0, &r1, &r2, &r3, &r4] {
        db.write_all_to_region(r, b"data")?;
    }

    // Remove pattern: keep, remove, keep, remove, keep
    // This creates 2 separate holes
    db.remove_region(r1)?;
    db.remove_region(r3)?;

    let layout = db.layout();
    assert_eq!(layout.start_to_hole().len(), 2);
    drop(layout);

    // Create a new region - should fill one of the holes
    let r5 = db.create_region_if_needed("r5")?;
    db.write_all_to_region(&r5, b"fills hole")?;

    let layout = db.layout();
    assert_eq!(layout.start_to_hole().len(), 1); // One hole filled, one remains

    Ok(())
}

#[test]
fn test_set_min_len_preallocate() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Preallocate large file
    let large_size = PAGE_SIZE * 1000;
    db.set_min_len(large_size)?;

    let file_len = db.file_len()?;
    assert!(file_len >= large_size);

    // Should still be able to write
    let region = db.create_region_if_needed("test")?;
    db.write_all_to_region(&region, b"After preallocation")?;

    Ok(())
}

// ============================================================================
// Data Integrity Tests
// ============================================================================

#[test]
fn test_partial_overwrites_data_integrity() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("partial")?;

    // Write initial pattern
    let initial = b"AAAAAAAAAA";
    db.write_all_to_region(&region, initial)?;

    // Overwrite middle
    db.write_all_to_region_at(&region, b"BBB", 3)?;

    // Overwrite start
    db.write_all_to_region_at(&region, b"CC", 0)?;

    // Overwrite end
    db.write_all_to_region_at(&region, b"DD", 8)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"CCABBBAADD");

    Ok(())
}

#[test]
fn test_write_at_exact_reserved_boundary() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Fill exactly to PAGE_SIZE
    let data = vec![42u8; PAGE_SIZE as usize];
    db.write_all_to_region(&region, &data)?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), PAGE_SIZE);
    assert_eq!(meta.reserved(), PAGE_SIZE);
    drop(meta);

    // Writing one more byte should trigger expansion
    db.write_all_to_region(&region, b"X")?;

    let meta = region.meta().read();
    assert_eq!(meta.len(), PAGE_SIZE + 1);
    assert!(meta.reserved() > PAGE_SIZE);

    Ok(())
}
