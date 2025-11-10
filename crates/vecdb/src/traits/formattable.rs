use std::fmt::{self, Display, Write};

use crate::likely;

pub trait Formattable: Display {
    /// Write the value in CSV format (with escaping if needed)
    #[inline]
    fn fmt_csv(&self, f: &mut String) -> fmt::Result {
        if likely(!Self::may_need_escaping()) {
            write!(f, "{}", self)?;
            return Ok(());
        }
        let start = f.len();
        write!(f, "{}", self)?;
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }

    /// Returns true if this type might need escaping
    fn may_need_escaping() -> bool;
}

// Implement for numeric types (no escaping needed)
macro_rules! impl_csv_display_numeric {
    ($($t:ty),*) => {
        $(
            impl Formattable for $t {
                fn may_need_escaping() -> bool {
                    false
                }
            }
        )*
    };
}

impl_csv_display_numeric!(
    bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);
