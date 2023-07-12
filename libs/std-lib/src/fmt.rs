pub type BoxedFmt<'lt> = Fmt<Box<dyn Fn(&mut core::fmt::Formatter) -> core::fmt::Result + 'lt>>;

pub struct Fmt<F>(pub F)
where
    F: Fn(&mut core::fmt::Formatter) -> core::fmt::Result;

impl<F> core::fmt::Display for Fmt<F>
where
    F: Fn(&mut core::fmt::Formatter) -> core::fmt::Result,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        (self.0)(f)
    }
}

impl<F> core::fmt::Debug for Fmt<F>
where
    F: Fn(&mut core::fmt::Formatter) -> core::fmt::Result,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        (self.0)(f)
    }
}

#[macro_export]
macro_rules! fmt {
    (type $lt: lifetime) => { $crate::fmt::Fmt<impl Fn(&mut core::fmt::Formatter) -> core::fmt::Result + $lt> };
    (type) => { $crate::fmt::Fmt<impl Fn(&mut core::fmt::Formatter) -> core::fmt::Result> };
}

pub use fmt;
