use std::{collections::BTreeSet, fs, path::Path};

use vecdb::{
    AnyStoredVec, AnyVec, CollectableVec, Database, GenericStoredVec, RawVec, Stamp,
    VecIteratorExtended, Version,
};

#[allow(clippy::upper_case_acronyms)]
type VEC = RawVec<usize, u32>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = fs::remove_dir_all("raw");

    let version = Version::TWO;

    let database = Database::open(Path::new("raw"))?;

    let mut options = (&database, "vec", version).into();

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        let mut iter = vec.into_iter();
        assert!(iter.get(0) == Some(0));
        assert!(iter.get(1) == Some(1));
        assert!(iter.get(2) == Some(2));
        assert!(iter.get(20) == Some(20));
        assert!(iter.get(21).is_none());
        drop(iter);

        vec.flush()?;

        assert!(vec.header().stamp() == Stamp::new(0));
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        vec.mut_header().update_stamp(Stamp::new(100));

        assert!(vec.header().stamp() == Stamp::new(100));

        let mut iter = vec.into_iter();
        assert!(iter.get(0) == Some(0));
        assert!(iter.get(1) == Some(1));
        assert!(iter.get(2) == Some(2));
        assert!(iter.get(3) == Some(3));
        assert!(iter.get(4) == Some(4));
        assert!(iter.get(5) == Some(5));
        assert!(iter.get(20) == Some(20));
        assert!(iter.get(20) == Some(20));
        assert!(iter.get(0) == Some(0));
        drop(iter);

        vec.push(21);
        vec.push(22);

        assert!(vec.stored_len() == 21);
        assert!(vec.pushed_len() == 2);
        assert!(vec.len() == 23);

        let mut iter = vec.into_iter();
        assert!(iter.get(20) == Some(20));
        assert!(iter.get(21) == Some(21));
        assert!(iter.get(22) == Some(22));
        assert!(iter.get(23).is_none());
        drop(iter);

        vec.flush()?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert!(vec.header().stamp() == Stamp::new(100));

        assert!(vec.stored_len() == 23);
        assert!(vec.pushed_len() == 0);
        assert!(vec.len() == 23);

        let mut iter = vec.into_iter();
        assert!(iter.get(0) == Some(0));
        assert!(iter.get(20) == Some(20));
        assert!(iter.get(21) == Some(21));
        assert!(iter.get(22) == Some(22));
        drop(iter);

        vec.truncate_if_needed(14)?;

        assert_eq!(vec.stored_len(), 14);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 14);

        let mut iter = vec.into_iter();
        assert_eq!(iter.get(0), Some(0));
        assert_eq!(iter.get(5), Some(5));
        assert_eq!(iter.get(20), None);
        drop(iter);

        assert_eq!(
            vec.collect_signed_range(Some(-5), None),
            vec![9, 10, 11, 12, 13]
        );

        vec.push(vec.len() as u32);
        assert_eq!(vec.iter()?.last(), Some(14));

        assert_eq!(
            vec.into_iter().collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.flush()?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert_eq!(vec.iter()?.last(), Some(14));

        assert_eq!(
            vec.into_iter().collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.reset()?;

        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 0);

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.pushed_len(), 21);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 21);

        let mut iter = vec.into_iter();
        assert_eq!(iter.get(0), Some(0));
        assert_eq!(iter.get(20), Some(20));
        assert!(iter.get(21).is_none());
        drop(iter);

        let reader = vec.create_static_reader();
        assert_eq!(vec.take(10, &reader)?, Some(10));
        assert_eq!(vec.holes(), &BTreeSet::from([10]));
        assert!(vec.get_any_or_read(10, &reader)?.is_none());
        drop(reader);

        vec.flush()?;

        assert!(vec.holes() == &BTreeSet::from([10]));
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert!(vec.holes() == &BTreeSet::from([10]));

        let reader = vec.create_static_reader();
        assert!(vec.get_any_or_read(10, &reader)?.is_none());
        drop(reader);

        vec.update(10, 10)?;
        vec.update(0, 10)?;

        let reader = vec.create_static_reader();
        assert_eq!(vec.holes(), &BTreeSet::new());
        assert_eq!(vec.get_any_or_read(0, &reader)?, Some(10));
        assert_eq!(vec.get_any_or_read(10, &reader)?, Some(10));
        drop(reader);

        vec.flush()?;
    }

    options = options.with_saved_stamped_changes(10);

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.truncate_if_needed(10)?;

        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.collect_holed()?,
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.stamped_flush_with_changes(Stamp::new(1))?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert_eq!(vec.collect(), vec![10, 1, 2, 5, 4]);

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.stamped_flush_with_changes(Stamp::new(2))?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.rollback()?;

        assert_eq!(vec.stamp(), Stamp::new(1));

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed()?,
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.rollback()?;

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_flush(Stamp::new(0))?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;

        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.collect_holed()?,
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.stamped_flush_with_changes(Stamp::new(1))?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert_eq!(vec.collect(), vec![10, 1, 2, 5, 4]);

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.stamped_flush_with_changes(Stamp::new(2))?;
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.rollback_before(Stamp::new(1))?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_flush(Stamp::new(0))?;

        vec.truncate_if_needed(10)?;
        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;
        dbg!(("0", vec.prev_holes(), vec.updated()));

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;
        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.stamped_flush_with_changes(Stamp::new(1))?;
        assert_eq!(vec.stamp(), Stamp::new(1));

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.stamped_flush_with_changes(Stamp::new(2))?;

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.rollback_before(Stamp::new(1))?;

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
        assert_eq!(vec.stamp(), Stamp::new(0));

        vec.truncate_if_needed(10)?;
        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        assert_eq!(vec.stamp(), Stamp::new(0));
        vec.stamped_flush_with_changes(Stamp::new(2))?;
        assert_eq!(vec.stamp(), Stamp::new(2));

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        dbg!("-----------------------------------------------------------------------");

        vec.rollback_before(Stamp::new(1))?;

        dbg!((
            vec.prev_stored_len(),
            vec.stored_len(),
            vec.real_stored_len(),
            vec.prev_pushed(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(vec.stamp(), Stamp::new(0));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_flush_with_changes(Stamp::new(0))?;

        let vec: VEC = RawVec::forced_import_with(options)?;
        dbg!(("0", vec.prev_holes(), vec.updated()));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
    }

    {
        let mut vec: VEC = RawVec::forced_import_with(options)?;

        dbg!(("0", vec.stamp(), vec.stored_len()));

        vec.truncate_if_needed(10)?;
        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        vec.stamped_flush_with_changes(Stamp::new(1))?;
        assert_eq!(vec.stamp(), Stamp::new(1));

        dbg!(("1", vec.stamp(), vec.stored_len()));

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        vec.stamped_flush_with_changes(Stamp::new(2))?;

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.rollback_before(Stamp::new(1))?;
        dbg!(("roll", vec.stamp(), vec.stored_len()));
        assert_eq!(vec.stamp(), Stamp::new(0));

        vec.truncate_if_needed(10)?;
        let reader = vec.create_static_reader();
        vec.take(5, &reader)?;
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.create_static_reader();
        vec.take(0, &reader)?;
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed()?,
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        dbg!(1);

        vec.rollback_before(Stamp::new(1))?;
        dbg!(1);

        assert_eq!(vec.stamp(), Stamp::new(0));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_flush_with_changes(Stamp::new(0))?;

        let vec: VEC = RawVec::forced_import_with(options)?;
        dbg!(("0", vec.prev_holes(), vec.updated()));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
    }

    Ok(())
}
