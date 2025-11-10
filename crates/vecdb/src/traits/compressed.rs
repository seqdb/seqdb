use pco::data_types::Number;

use super::StoredRaw;

pub trait TransparentStoredCompressed<T> {}

pub trait StoredCompressed
where
    Self: StoredRaw + Copy + 'static + TransparentStoredCompressed<Self::NumberType>,
{
    type NumberType: pco::data_types::Number;
}

pub trait AsInnerSlice<T>
where
    T: Number,
{
    fn as_inner_slice(&self) -> &[T];
}

impl<T> AsInnerSlice<T::NumberType> for [T]
where
    T: StoredCompressed,
{
    fn as_inner_slice(&self) -> &[T::NumberType] {
        assert_eq!(
            std::mem::size_of::<T>(),
            std::mem::size_of::<T::NumberType>()
        );
        assert_eq!(
            std::mem::align_of::<T>(),
            std::mem::align_of::<T::NumberType>()
        );
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const T::NumberType, self.len()) }
    }
}

pub trait FromInnerSlice<T> {
    fn from_inner_slice(slice: Vec<T>) -> Vec<Self>
    where
        Self: Sized;
}

impl<T> FromInnerSlice<T::NumberType> for T
where
    T: StoredCompressed,
{
    fn from_inner_slice(vec: Vec<T::NumberType>) -> Vec<T> {
        assert_eq!(
            std::mem::size_of::<T>(),
            std::mem::size_of::<T::NumberType>()
        );
        assert_eq!(
            std::mem::align_of::<T>(),
            std::mem::align_of::<T::NumberType>()
        );

        let mut vec = std::mem::ManuallyDrop::new(vec);
        unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut T, vec.len(), vec.capacity()) }
    }
}

macro_rules! impl_stored_compressed {
    ($($t:ty),*) => {
        $(
            impl
TransparentStoredCompressed<$t> for $t {}
            impl StoredCompressed for $t {
                type NumberType = $t;
            }
        )*
    };
}

impl_stored_compressed!(u16, u32, u64, i16, i32, i64, f32, f64);
