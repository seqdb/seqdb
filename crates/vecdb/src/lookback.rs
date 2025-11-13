use std::collections::VecDeque;

use crate::{BoxedVecIterator, VecIndex, VecValue};

/// Windowed lookback access - either using a ring buffer or direct access
pub enum Lookback<'a, I: VecIndex, T: VecValue> {
    Buffer {
        window: usize,
        buf: VecDeque<T>,
    },
    DirectAccess {
        window: usize,
        iter: BoxedVecIterator<'a, I, T>,
    },
}

impl<'a, I: VecIndex, T: VecValue> Lookback<'a, I, T> {
    /// Get the value at lookback position, returning default if not enough history.
    /// For Buffer: returns front of buffer if full (keeps window), otherwise returns default.
    /// For DirectAccess: uses get_at_unwrap at index-window.
    pub fn get_at_lookback(&mut self, index: usize, default: T) -> T
    where
        T: Default + Clone,
    {
        match self {
            Self::Buffer { window, buf } => {
                if buf.len() > *window {
                    buf.front().cloned().unwrap()
                } else {
                    default
                }
            }
            Self::DirectAccess { window, iter } => index
                .checked_sub(*window)
                .map(|prev_i| iter.get_at_unwrap(prev_i))
                .unwrap_or(default),
        }
    }

    /// Get the value at lookback position and add current value to buffer.
    /// For Buffer: pops front if at capacity, then pushes current value.
    /// Returns the lookback value or default if not enough history.
    pub fn get_and_push(&mut self, index: usize, current: T, default: T) -> T
    where
        T: Clone,
    {
        match self {
            Self::Buffer { window, buf } => {
                let val = if buf.len() == *window {
                    buf.pop_front().unwrap()
                } else {
                    default
                };
                buf.push_back(current);
                val
            }
            Self::DirectAccess { window, iter } => index
                .checked_sub(*window)
                .map(|prev_i| iter.get_at_unwrap(prev_i))
                .unwrap_or(default),
        }
    }

    /// Push current value and maintain window size for Buffer strategy.
    /// For DirectAccess: no-op.
    pub fn push_and_maintain(&mut self, current: T)
    where
        T: Clone,
    {
        if let Self::Buffer { window, buf } = self {
            buf.push_back(current);
            if buf.len() > *window + 1 {
                buf.pop_front();
            }
        }
    }
}
