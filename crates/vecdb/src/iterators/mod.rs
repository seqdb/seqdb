mod boxed;
mod enumerated;
mod extended;

pub use boxed::*;
pub use enumerated::*;
pub use extended::*;

pub trait VecIterator: Iterator {
    /// Skip n elements efficiently (may use file seeking instead of iteration).
    /// Returns Self to avoid wrapper types.
    fn skip_optimized(self, n: usize) -> Self
    where
        Self: Sized;

    /// Take n elements efficiently (may adjust internal bounds).
    /// Returns Self to avoid wrapper types.
    fn take_optimized(self, n: usize) -> Self
    where
        Self: Sized;

    /// Efficiently skip and take in one operation.
    fn slice_optimized(self, start: usize, len: usize) -> Self
    where
        Self: Sized,
    {
        self.skip_optimized(start).take_optimized(len)
    }

    /// Optimized Enumerate with inner optimized iterator functions
    fn enumerate_optimized(self) -> Enumerated<Self>
    where
        Self: Sized,
    {
        Enumerated::new(self, 0)
    }
}
