use std::{fmt::Write, marker::PhantomData};

use crate::{Result, StoredRaw};

/// A stateful writer that can write one value at a time to a buffer
pub trait ValueWriter {
    /// Write the next value to the buffer, returns false if no more values
    fn write_next(&mut self, buf: &mut String) -> Result<bool>;
}

pub struct VecIteratorWriter<'a, I, T> {
    pub iter: Box<dyn Iterator<Item = T> + 'a>,
    pub _phantom: PhantomData<I>,
}

impl<'a, I, T> ValueWriter for VecIteratorWriter<'a, I, T>
where
    T: StoredRaw,
{
    fn write_next(&mut self, buf: &mut String) -> Result<bool> {
        if let Some(value) = self.iter.next() {
            write!(buf, "{}", value)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
