use databuf::{Decode, Result};

// ----------------------------------------------------------------------

pub trait FirstArg<'de, State>: Sized {
    fn decode(state: State, _: &mut &'de [u8]) -> Result<Self>;
}

impl<'de, State, Args> FirstArg<'de, State> for Args
where
    Args: Decode<'de>,
{
    fn decode(_: State, data: &mut &'de [u8]) -> Result<Self> {
        Args::decode::<{ crate::DATABUF_CONFIG }>(data)
    }
}

// ----------------------------------------------------------------------

pub trait Input<'de, State>: Sized {
    fn decode(state: State, _: &mut &'de [u8]) -> Result<Self>;
}

macro_rules! args_with_ctx {
    [$(($($name: ident)*))*] => {
        $(
            impl<'de, State, T0, $($name,)*> Input<'de, State> for (T0, $($name,)*)
            where
                T0: FirstArg<'de, State>,
                $($name: Decode<'de>,)*
            {
                fn decode(state: State, data: &mut &'de [u8]) -> Result<Self> {
                    Ok((
                        T0::decode(state, data)?,
                        $($name::decode::<{ crate::DATABUF_CONFIG }>(data)?,)*
                    ))
                }
            }
        )*
    };
}

impl<State> Input<'_, State> for () {
    fn decode(_: State, _: &mut &[u8]) -> Result<Self> {
        Ok(())
    }
}

args_with_ctx! {
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
}
