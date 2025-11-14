pub trait SaturatingAdd<Rhs = Self>: Sized {
    fn saturating_add(self, rhs: Rhs) -> Self;
}
