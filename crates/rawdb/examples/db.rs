use std::{fs, path::Path, sync::Arc};

use rawdb::{Database, PAGE_SIZE, Result};

fn main() -> Result<()> {
    let _ = fs::remove_dir_all("vecs");

    let db = Database::open(Path::new("vecs"))?;

    let region1 = db.create_region_if_needed("region1")?;

    {
        let layout = db.layout();

        assert!(layout.start_to_region().len() == 1);

        assert!(layout.start_to_hole().is_empty());

        let regions = db.regions();

        assert!(
            regions
                .get_region_from_id("region1")
                .is_some_and(|r| Arc::ptr_eq(r, &region1))
        );

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);
    }

    db.write_all_to_region(&region1, &[0, 1, 2, 3, 4])?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 5);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(db.mmap()[0..10] == [0, 1, 2, 3, 4, 0, 0, 0, 0, 0]);
    }

    db.write_all_to_region(&region1, &[5, 6, 7, 8, 9])?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(db.mmap()[0..10] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    db.write_all_to_region_at(&region1, &[1, 2], 0)?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(db.mmap()[0..10] == [1, 2, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    db.write_all_to_region_at(&region1, &[10, 11, 12, 13, 14, 15, 16, 17, 18], 4)?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 13);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            db.mmap()[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 0, 0
                ]
        );
    }

    db.write_all_to_region_at(&region1, &[0, 0, 0, 0, 0, 1], 13)?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 19);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            db.mmap()[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 1, 0
                ]
        );
    }

    db.write_all_to_region_at(&region1, &[1; 8000], 0)?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);
        assert!(db.mmap()[0..8000] == [1; 8000]);
        assert!(db.mmap()[8000..8001] == [0]);
    }

    println!("Disk usage - pre sync: {}", db.disk_usage());

    db.flush()?;

    println!("Disk usage - post sync: {}", db.disk_usage());

    db.truncate_region(&region1, 10)?;
    db.compact()?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);

        // We only punch a hole in whole pages (4096 bytes)
        // Thus the last byte of the page where the is still data wasn't overwritten when truncating
        // And the first byte of the punched page was set to 0
        assert!(db.mmap()[4095..=4096] == [1, 0]);
    }

    println!("Disk usage - pre sync: {}", db.disk_usage());
    db.flush()?;
    println!("Disk usage - post sync: {}", db.disk_usage());

    db.truncate_region(&region1, 10)?;
    db.compact()?;

    {
        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);
        // We only punch a hole in whole pages (4096 bytes)
        // Thus the last byte of the page where the is still data wasn't overwritten when truncating
        // And the first byte of the punched page was set to 0
        assert!(db.mmap()[4095..=4096] == [1, 0]);
    }

    db.flush()?;
    println!("Disk usage - post trunc: {}", db.disk_usage());

    db.remove_region(region1)?;
    db.compact()?;

    println!("Disk usage - post remove: {}", db.disk_usage());

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 1);
        assert!(index_to_region[0].is_none());
        assert!(regions.id_to_index().is_empty());

        let layout = db.layout();
        assert!(layout.start_to_region().is_empty());
        assert!(layout.start_to_hole().len() == 1);
    }

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let region3 = db.create_region_if_needed("region3")?;

    // dbg!(db.layout());

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);

        let region2_meta = region2.meta().read();
        assert!(region2_meta.start() == PAGE_SIZE);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta().read();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 3);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 3);

        assert!(Arc::ptr_eq(start_to_index.get(&0).unwrap(), &region1));
        assert!(Arc::ptr_eq(
            start_to_index.get(&PAGE_SIZE).unwrap(),
            &region2
        ));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 2)).unwrap(),
            &region3
        ));
        assert!(layout.start_to_hole().is_empty());
    }

    db.remove_region(region2)?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(index_to_region.get(1).is_some_and(|opt| opt.is_none()));

        let region3_meta = region3.meta().read();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(Arc::ptr_eq(start_to_index.get(&0).unwrap(), &region1));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 2)).unwrap(),
            &region3
        ));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));
    }

    let region2 = db.create_region_if_needed("region2")?;
    let region2_i = region2.index();
    assert!(region2_i == 1);

    db.remove_region(region2)?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );

        let region3_meta = region3.meta().read();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(Arc::ptr_eq(start_to_index.get(&0).unwrap(), &region1));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 2)).unwrap(),
            &region3
        ));

        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));
    }

    db.write_all_to_region_at(&region1, &[1; 8000], 0)?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );

        let region3_meta = region3.meta().read();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(Arc::ptr_eq(start_to_index.get(&0).unwrap(), &region1));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 2)).unwrap(),
            &region3
        ));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    let region2 = db.create_region_if_needed("region2")?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region2_meta = region2.meta().read();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta().read();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 3);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 3);
        assert!(Arc::ptr_eq(start_to_index.get(&0).unwrap(), &region1));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 2)).unwrap(),
            &region3
        ));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 3)).unwrap(),
            &region2
        ));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    db.remove_region(region3)?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region2_meta = region2.meta().read();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3").is_none());

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(Arc::ptr_eq(start_to_index.get(&0).unwrap(), &region1));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 3)).unwrap(),
            &region2
        ));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&(PAGE_SIZE * 2)) == Some(&PAGE_SIZE));
    }

    db.write_all_to_region(&region1, &[1; 8000])?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta().read();
        assert!(region1_meta.start() == PAGE_SIZE * 4);
        assert!(region1_meta.len() == 16_000);
        assert!(region1_meta.reserved() == 4 * PAGE_SIZE);

        let region2_meta = region2.meta().read();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3").is_none());

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 4)).unwrap(),
            &region1
        ));
        assert!(Arc::ptr_eq(
            start_to_index.get(&(PAGE_SIZE * 3)).unwrap(),
            &region2
        ));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&0) == Some(&(PAGE_SIZE * 3)));
    }

    db.write_all_to_region(&region2, &[1; 6000])?;

    let region4 = db.create_region_if_needed("region4")?;
    db.remove_region(region2)?;
    db.remove_region(region4)?;

    let regions = db.regions();
    dbg!(&regions);
    let layout = db.layout();
    dbg!(&layout);

    Ok(())
}
