use super::*;
use std::{rc::Rc, sync::Arc};

// impl TypeId for std::convert::Infallible {
//     fn ty() -> Ty {
//         Ty::Never
//     }
// }

macro_rules! impl_type_id_for {
    [$($ty: ty), *] => {$(
        impl<T: TypeId> TypeId for $ty {
            fn ty(c: &mut CostomTypes) -> Ty {
                T::ty(c)
            }
        }
    )*};
}

impl_type_id_for! { &T, Box<T>, Arc<T>, Rc<T> }

macro_rules! impl_for_typles {
    [$(($($ty: ident)*))*]  => ($(
        impl<$($ty),*> TypeId for ($($ty,)*)
        where
            $($ty: TypeId),*
        {
            fn ty(_c: &mut CostomTypes) -> Ty {
                Ty::Tuple(vec![$($ty::ty(_c)),*])
            }
        }
    )*);
}

impl_for_typles!(
    ()
    (T1)
    (T1 T2)
    (T1 T2 T3)
    (T1 T2 T3 T4)
    (T1 T2 T3 T4 T5)
    (T1 T2 T3 T4 T5 T6)
    (T1 T2 T3 T4 T5 T6 T7)
    (T1 T2 T3 T4 T5 T6 T7 T8)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16)
);
