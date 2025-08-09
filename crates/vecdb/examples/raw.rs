use std::{borrow::Cow, collections::BTreeSet, fs, path::Path};

use vecdb::{
    AnyStoredVec, AnyVec, CollectableVec, Database, GenericStoredVec, RawVec, Stamp, VecIterator,
    Version,
};

#[allow(clippy::upper_case_acronyms)]
type VEC = RawVec<usize, u32>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = fs::remove_dir_all("raw");

    let version = Version::TWO;

    let database = Database::open(Path::new("raw"))?;

    {
        let mut vec: VEC = RawVec::forced_import(&database, "vec", version)?;

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        let mut iter = vec.into_iter();
        assert!(iter.get(0) == Some(Cow::Borrowed(&0)));
        assert!(iter.get(1) == Some(Cow::Borrowed(&1)));
        assert!(iter.get(2) == Some(Cow::Borrowed(&2)));
        assert!(iter.get(20) == Some(Cow::Borrowed(&20)));
        assert!(iter.get(21).is_none());
        drop(iter);

        vec.flush()?;

        assert!(vec.header().stamp() == Stamp::new(0));
    }

    {
        let mut vec: VEC = RawVec::forced_import(&database, "vec", version)?;

        vec.mut_header().update_stamp(Stamp::new(100));

        assert!(vec.header().stamp() == Stamp::new(100));

        let mut iter = vec.into_iter();
        assert!(iter.get(0) == Some(Cow::Borrowed(&0)));
        assert!(iter.get(1) == Some(Cow::Borrowed(&1)));
        assert!(iter.get(2) == Some(Cow::Borrowed(&2)));
        assert!(iter.get(3) == Some(Cow::Borrowed(&3)));
        assert!(iter.get(4) == Some(Cow::Borrowed(&4)));
        assert!(iter.get(5) == Some(Cow::Borrowed(&5)));
        assert!(iter.get(20) == Some(Cow::Borrowed(&20)));
        assert!(iter.get(20) == Some(Cow::Borrowed(&20)));
        assert!(iter.get(0) == Some(Cow::Borrowed(&0)));
        drop(iter);

        vec.push(21);
        vec.push(22);

        assert!(vec.stored_len() == 21);
        assert!(vec.pushed_len() == 2);
        assert!(vec.len() == 23);

        let mut iter = vec.into_iter();
        assert!(iter.get(20) == Some(Cow::Borrowed(&20)));
        assert!(iter.get(21) == Some(Cow::Borrowed(&21)));
        assert!(iter.get(22) == Some(Cow::Borrowed(&22)));
        assert!(iter.get(23).is_none());
        drop(iter);

        vec.flush()?;
    }

    {
        let mut vec: VEC = RawVec::forced_import(&database, "vec", version)?;

        assert!(vec.header().stamp() == Stamp::new(100));

        assert!(vec.stored_len() == 23);
        assert!(vec.pushed_len() == 0);
        assert!(vec.len() == 23);

        let mut iter = vec.into_iter();
        assert!(iter.get(0) == Some(Cow::Borrowed(&0)));
        assert!(iter.get(20) == Some(Cow::Borrowed(&20)));
        assert!(iter.get(21) == Some(Cow::Borrowed(&21)));
        assert!(iter.get(22) == Some(Cow::Borrowed(&22)));
        drop(iter);

        vec.truncate_if_needed(14)?;

        assert_eq!(vec.stored_len(), 14);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 14);

        let mut iter = vec.into_iter();
        assert_eq!(iter.get(0), Some(Cow::Borrowed(&0)));
        assert_eq!(iter.get(5), Some(Cow::Borrowed(&5)));
        assert_eq!(iter.get(20), None);
        drop(iter);

        assert_eq!(
            vec.collect_signed_range(Some(-5), None)?,
            vec![9, 10, 11, 12, 13]
        );

        vec.push(vec.len() as u32);
        assert_eq!(
            VecIterator::last(vec.into_iter()),
            Some((14, Cow::Borrowed(&14)))
        );

        assert_eq!(
            vec.into_iter()
                .map(|(_, v)| v.into_owned())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.flush()?;
    }

    {
        let mut vec: VEC = RawVec::forced_import(&database, "vec", version)?;

        assert_eq!(
            VecIterator::last(vec.into_iter()),
            Some((14, Cow::Borrowed(&14)))
        );

        assert_eq!(
            vec.into_iter()
                .map(|(_, v)| v.into_owned())
                .collect::<Vec<_>>(),
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
        assert_eq!(iter.get(0), Some(Cow::Borrowed(&0)));
        assert_eq!(iter.get(20), Some(Cow::Borrowed(&20)));
        assert!(iter.get(21).is_none());
        drop(iter);

        let reader = vec.create_static_reader();
        assert_eq!(vec.take(10, &reader)?, Some(10));
        assert_eq!(vec.holes(), &BTreeSet::from([10]));
        assert!(vec.get_or_read(10, &reader)?.is_none());
        drop(reader);

        vec.flush()?;

        assert!(vec.holes() == &BTreeSet::from([10]));
    }

    {
        let mut vec: VEC = RawVec::forced_import(&database, "vec", version)?;

        assert!(vec.holes() == &BTreeSet::from([10]));

        let reader = vec.create_static_reader();
        assert!(vec.get_or_read(10, &reader)?.is_none());
        drop(reader);

        vec.update(10, 10)?;
        vec.update(0, 10)?;

        let reader = vec.create_static_reader();
        assert_eq!(vec.holes(), &BTreeSet::new());
        assert_eq!(vec.get_or_read(0, &reader)?, Some(Cow::Borrowed(&10)));
        assert_eq!(vec.get_or_read(10, &reader)?, Some(Cow::Borrowed(&10)));
        drop(reader);

        vec.flush()?;
    }

    {
        let vec: VEC = RawVec::forced_import(&database, "vec", version)?;

        assert_eq!(
            vec.collect()?,
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
    }

    Ok(())
}
