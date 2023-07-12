use super::*;

macro_rules! impl_for {
    [$($ty:tt),*] => {$(impl TypeId for $ty { fn ty(_: &mut CostomTypes) -> Ty { Ty::$ty } })*};
}

impl_for!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, bool, /* char, */ String);

impl TypeId for usize {
    fn ty(_: &mut CostomTypes) -> Ty {
        match usize::BITS {
            32 => Ty::u32,
            64 => Ty::u64,
            _ => Ty::u16,
        }
    }
}

impl TypeId for isize {
    fn ty(_: &mut CostomTypes) -> Ty {
        match isize::BITS {
            32 => Ty::i32,
            64 => Ty::i64,
            _ => Ty::i16,
        }
    }
}

impl<T: TypeId, const N: usize> TypeId for [T; N] {
    fn ty(def: &mut CostomTypes) -> Ty {
        Ty::Array {
            len: N,
            ty: Box::new(T::ty(def)),
        }
    }
}

impl<T: TypeId> TypeId for Option<T> {
    fn ty(def: &mut CostomTypes) -> Ty {
        Ty::Option(Box::new(T::ty(def)))
    }
}

impl<T: TypeId, E: TypeId> TypeId for Result<T, E> {
    fn ty(def: &mut CostomTypes) -> Ty {
        Ty::Result(Box::new((T::ty(def), E::ty(def))))
    }
}

impl TypeId for &str {
    fn ty(_: &mut CostomTypes) -> Ty {
        Ty::String
    }
}
