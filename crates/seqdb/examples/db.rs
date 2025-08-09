use std::{fs, path::Path};

use seqdb::{Database, PAGE_SIZE, Result};

fn main() -> Result<()> {
    let _ = fs::remove_dir_all("vecs");

    let database = Database::open(Path::new("vecs"))?;

    // let seqdb_min_len = PAGE_SIZE * 1_000_000;
    // let min_regions = 20_000;

    // seqdb.set_min_len(seqdb_min_len)?;
    // seqdb.set_min_regions(min_regions)?;

    let (region1_i, _) = database.create_region_if_needed("region1")?;

    {
        let layout = database.layout();
        assert!(layout.start_to_index().len() == 1);
        assert!(layout.start_to_index().first_key_value() == Some((&0, &0)));
        assert!(layout.start_to_hole().is_empty());

        let regions = database.regions();
        assert!(
            regions
                .get_region_index_from_id("region1")
                .is_some_and(|i| i == region1_i)
        );

        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 0);
        assert!(region.reserved() == PAGE_SIZE);
    }

    database.write_all_to_region(region1_i.into(), &[0, 1, 2, 3, 4])?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 5);
        assert!(region.reserved() == PAGE_SIZE);

        assert!(database.mmap()[0..10] == [0, 1, 2, 3, 4, 0, 0, 0, 0, 0]);
    }

    database.write_all_to_region(region1_i.into(), &[5, 6, 7, 8, 9])?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 10);
        assert!(region.reserved() == PAGE_SIZE);

        assert!(database.mmap()[0..10] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    database.write_all_to_region_at(region1_i.into(), &[1, 2], 0)?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 10);
        assert!(region.reserved() == PAGE_SIZE);

        assert!(database.mmap()[0..10] == [1, 2, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    database.write_all_to_region_at(region1_i.into(), &[10, 11, 12, 13, 14, 15, 16, 17, 18], 4)?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 13);
        assert!(region.reserved() == PAGE_SIZE);

        assert!(
            database.mmap()[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 0, 0
                ]
        );
    }

    database.write_all_to_region_at(region1_i.into(), &[0, 0, 0, 0, 0, 1], 13)?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 19);
        assert!(region.reserved() == PAGE_SIZE);

        assert!(
            database.mmap()[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 1, 0
                ]
        );
    }

    dbg!(1);

    database.write_all_to_region_at(region1_i.into(), &[1; 8000], 0)?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 8000);
        assert!(region.reserved() == PAGE_SIZE * 2);

        assert!(database.mmap()[0..8000] == [1; 8000]);
        assert!(database.mmap()[8000..8001] == [0]);
    }

    println!("Disk usage - pre sync: {}", database.disk_usage());
    database.flush()?;
    println!("Disk usage - post sync: {}", database.disk_usage());

    database.truncate_region(region1_i.into(), 10)?;
    database.punch_holes()?;

    {
        let region = database.get_region(region1_i.into())?;
        assert!(region.start() == 0);
        assert!(region.len() == 10);
        assert!(region.reserved() == PAGE_SIZE * 2);
        // We only punch a hole in whole pages (4096 bytes)
        // Thus the last byte of the page where the is still data wasn't overwritten when truncating
        // And the first byte of the punched page was set to 0
        assert!(database.mmap()[4095..=4096] == [1, 0]);
    }

    database.flush()?;
    println!("Disk usage - post trunc: {}", database.disk_usage());

    database.remove_region(region1_i.into())?;

    database.flush()?;

    println!("Disk usage - post remove: {}", database.disk_usage());

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 1);
        assert!(index_to_region[0].is_none());
        assert!(regions.id_to_index().is_empty());

        let layout = database.layout();
        assert!(layout.start_to_index().is_empty());
        assert!(layout.start_to_hole().len() == 1);
    }

    let (region1_i, _) = database.create_region_if_needed("region1")?;
    let (region2_i, _) = database.create_region_if_needed("region2")?;
    let (region3_i, _) = database.create_region_if_needed("region3")?;

    // dbg!(seqdb.layout());

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == 0);
        assert!(region1.len() == 0);
        assert!(region1.reserved() == PAGE_SIZE);
        let region2 = database.get_region(region2_i.into())?;
        assert!(region2.start() == PAGE_SIZE);
        assert!(region2.len() == 0);
        assert!(region2.reserved() == PAGE_SIZE);
        let region3 = database.get_region(region3_i.into())?;
        assert!(region3.start() == PAGE_SIZE * 2);
        assert!(region3.len() == 0);
        assert!(region3.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 3);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 3);
        assert!(start_to_index.get(&0) == Some(&0));
        assert!(start_to_index.get(&PAGE_SIZE) == Some(&1));
        assert!(start_to_index.get(&(PAGE_SIZE * 2)) == Some(&2));
        assert!(layout.start_to_hole().is_empty());
    }

    database.remove_region(region2_i.into())?;

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == 0);
        assert!(region1.len() == 0);
        assert!(region1.reserved() == PAGE_SIZE);
        assert!(database.get_region(region2_i.into()).is_err());
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );
        let region3 = database.get_region(region3_i.into())?;
        assert!(region3.start() == PAGE_SIZE * 2);
        assert!(region3.len() == 0);
        assert!(region3.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0) == Some(&region1_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 2)) == Some(&region3_i));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));

        drop(regions);
        drop(layout);
        assert!(
            database
                .remove_region(region2_i.into())
                .is_ok_and(|o| o.is_none())
        );
    }

    let (region2_i, _) = database.create_region_if_needed("region2")?;

    {
        assert!(region2_i == 1)
    }

    database.remove_region(region2_i.into())?;

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == 0);
        assert!(region1.len() == 0);
        assert!(region1.reserved() == PAGE_SIZE);
        assert!(database.get_region(region2_i.into()).is_err());
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );
        let region3 = database.get_region(region3_i.into())?;
        assert!(region3.start() == PAGE_SIZE * 2);
        assert!(region3.len() == 0);
        assert!(region3.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0) == Some(&region1_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 2)) == Some(&region3_i));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));

        drop(regions);
        drop(layout);
        assert!(
            database
                .remove_region(region2_i.into())
                .is_ok_and(|o| o.is_none())
        );
    }

    database.write_all_to_region_at(region1_i.into(), &[1; 8000], 0)?;

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == 0);
        assert!(region1.len() == 8000);
        assert!(region1.reserved() == 2 * PAGE_SIZE);
        assert!(database.get_region(region2_i.into()).is_err());
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );
        let region3 = database.get_region(region3_i.into())?;
        assert!(region3.start() == PAGE_SIZE * 2);
        assert!(region3.len() == 0);
        assert!(region3.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0) == Some(&region1_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 2)) == Some(&region3_i));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    let (region2_i, _) = database.create_region_if_needed("region2")?;

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == 0);
        assert!(region1.len() == 8000);
        assert!(region1.reserved() == 2 * PAGE_SIZE);
        let region2 = database.get_region(region2_i.into())?;
        assert!(region2.start() == PAGE_SIZE * 3);
        assert!(region2.len() == 0);
        assert!(region2.reserved() == PAGE_SIZE);
        let region3 = database.get_region(region3_i.into())?;
        assert!(region3.start() == PAGE_SIZE * 2);
        assert!(region3.len() == 0);
        assert!(region3.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 3);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 3);
        assert!(start_to_index.get(&0) == Some(&region1_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 2)) == Some(&region3_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 3)) == Some(&region2_i));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    database.remove_region(region3_i.into())?;

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == 0);
        assert!(region1.len() == 8000);
        assert!(region1.reserved() == 2 * PAGE_SIZE);
        let region2 = database.get_region(region2_i.into())?;
        assert!(region2.start() == PAGE_SIZE * 3);
        assert!(region2.len() == 0);
        assert!(region2.reserved() == PAGE_SIZE);
        assert!(database.get_region(region3_i.into()).is_err());
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3").is_none());

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0) == Some(&region1_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 3)) == Some(&region2_i));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&(PAGE_SIZE * 2)) == Some(&PAGE_SIZE));
    }

    database.write_all_to_region(region1_i.into(), &[1; 8000])?;

    {
        let regions = database.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);
        let region1 = database.get_region(region1_i.into())?;
        assert!(region1.start() == PAGE_SIZE * 4);
        assert!(region1.len() == 16_000);
        assert!(region1.reserved() == 4 * PAGE_SIZE);
        let region2 = database.get_region(region2_i.into())?;
        assert!(region2.start() == PAGE_SIZE * 3);
        assert!(region2.len() == 0);
        assert!(region2.reserved() == PAGE_SIZE);
        assert!(database.get_region(region3_i.into()).is_err());
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3").is_none());

        let layout = database.layout();
        let start_to_index = layout.start_to_index();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&(PAGE_SIZE * 4)) == Some(&region1_i));
        assert!(start_to_index.get(&(PAGE_SIZE * 3)) == Some(&region2_i));
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&0) == Some(&(PAGE_SIZE * 3)));
    }

    database.write_all_to_region(region2_i.into(), &[1; 6000])?;

    let (region4_i, _) = database.create_region_if_needed("region4")?;
    database.remove_region(region2_i.into())?;
    database.remove_region(region4_i.into())?;

    let regions = database.regions();
    dbg!(&regions);
    let layout = database.layout();
    dbg!(&layout);

    Ok(())
}
