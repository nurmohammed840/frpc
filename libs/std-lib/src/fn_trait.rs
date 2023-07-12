pub trait FnOnce<Args> {
    type Output;
    fn call_once(self, args: Args) -> Self::Output;
}

pub trait FnMut<Args>: FnOnce<Args> {
    fn call_mut(&mut self, args: Args) -> Self::Output;
}

pub trait Fn<Args>: FnMut<Args> {
    fn call(&self, args: Args) -> Self::Output;
}

macro_rules! impl_for_typles {
    [$(($($i: tt: $ty: ident)*))*]  => ($(
        impl<Func, Ret, $($ty,)*> FnOnce<($($ty,)*)> for Func
        where
        Func: core::ops::FnOnce($($ty),*) -> Ret
        {
            type Output = Ret;
            #[inline] fn call_once(self, _args: ($($ty,)*)) -> Self::Output {
                self($(_args.$i),*)
            }
        }

        impl<Func, Ret, $($ty,)*> FnMut<($($ty,)*)> for Func
        where
            Func: core::ops::FnMut($($ty),*) -> Ret,
        {
            #[inline] fn call_mut(&mut self, _args: ($($ty,)*)) -> Self::Output {
                self($(_args.$i),*)
            }
        }

        impl<Func, Ret, $($ty,)*> Fn<($($ty,)*)> for Func
        where
            Func: core::ops::Fn($($ty),*) -> Ret,
        {
            #[inline] fn call(&self, _args: ($($ty,)*)) -> Self::Output {
                self($(_args.$i),*)
            }
        }
    )*);
}

impl_for_typles!(
    ()
    (0: T0)
    (0: T0 1: T1)
    (0: T0 1: T1 2: T2)
    (0: T0 1: T1 2: T2 3: T3)
    (0: T0 1: T1 2: T2 3: T3 4: T4)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11 12: T12)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11 12: T12 13: T13)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11 12: T12 13: T13 14: T14)
    (0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11 12: T12 13: T13 14: T14 15: T15)
);
