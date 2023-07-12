#![doc(hidden)]
use crate::output_type::OutputType;
pub use frpc_message;
use frpc_message::{CostomTypes, Func, Ty, TypeId};

pub fn fn_sig<F, Args>(
    _: &F,
    costom_types: &mut CostomTypes,
    index: u16,
    ident: &str,
    docs: &str,
) -> Func
where
    F: std_lib::FnOnce<Args>,
    Args: TypeId,
    F::Output: OutputType,
{
    let Ty::Tuple(mut args) = Args::ty(costom_types) else { unreachable!() };
    if let Some(ty) = args.first() {
        if ty.is_empty_tuple() {
            args.remove(0);
        }
    }
    Func {
        index,
        ident: frpc_message::Ident(ident.to_string()),
        args,
        output: <F::Output as OutputType>::fn_output_ty(costom_types),
        docs: docs.to_string(),
    }
}

impl<T> TypeId for crate::State<T> {
    fn ty(_: &mut CostomTypes) -> frpc_message::Ty {
        Ty::Tuple(vec![])
    }
}
