use std::marker::PhantomData;

use crate::{Error, Formattable, Result, VecValue};

/// A stateful writer that can write one value at a time to a buffer
pub trait ValueWriter {
    /// Write the next value to the buffer, returns false if no more values
    fn write_next(&mut self, buf: &mut String) -> Result<()>;
}

pub struct VecIteratorWriter<'a, I, T> {
    pub iter: Box<dyn Iterator<Item = T> + 'a>,
    pub _phantom: PhantomData<I>,
}

impl<'a, I, T> ValueWriter for VecIteratorWriter<'a, I, T>
where
    T: VecValue + Formattable,
{
    fn write_next(&mut self, buf: &mut String) -> Result<()> {
        if let Some(value) = self.iter.next() {
            value.fmt_csv(buf)?;
            Ok(())
        } else {
            Err(Error::WrongLength)
        }
    }
}
