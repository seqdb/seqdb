use rawdb::Database;
use tempfile::TempDir;
use vecdb::{AnyStoredVec, BytesVec, ImportableVec, Result, Version, WritableVec};

#[test]
fn raw_reader_cursor_reads_persisted_values() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let mut vec = BytesVec::<usize, u64>::import(&db, "values", Version::ONE)?;

    for value in 0..10 {
        vec.push(value);
    }
    vec.write()?;

    // Reader cursors intentionally do not include later uncommitted values.
    vec.push(10);
    assert_eq!(vec.read_once(9)?, 9);
    assert!(vec.read_once(10).is_err());
    let reader = vec.create_reader();
    assert!(vec.read_at(10, &reader).is_err());

    let reader = vec.reader();
    assert_eq!(vec.get_pushed_or_read(9, &reader), Some(9));
    assert_eq!(vec.get_pushed_or_read(10, &reader), Some(10));
    assert_eq!(vec.get_pushed_or_read(11, &reader), None);

    let mut cursor = vec.reader().cursor();
    assert_eq!(cursor.position(), 0);
    assert_eq!(cursor.remaining(), 10);
    assert_eq!(cursor.get(7), Some(7));
    assert_eq!(cursor.position(), 0);

    cursor.advance(3);
    assert_eq!(cursor.position(), 3);
    assert_eq!(cursor.next(), Some(3));
    assert_eq!(cursor.position(), 4);

    assert_eq!(
        cursor.fold(3, Vec::new(), |mut values, value| {
            values.push(value);
            values
        }),
        vec![4, 5, 6]
    );
    assert_eq!(cursor.position(), 7);

    let mut tail = Vec::new();
    cursor.for_each(usize::MAX, |value| tail.push(value));
    assert_eq!(tail, vec![7, 8, 9]);
    assert_eq!(cursor.position(), 10);
    assert_eq!(cursor.remaining(), 0);
    assert_eq!(cursor.next(), None);

    Ok(())
}
