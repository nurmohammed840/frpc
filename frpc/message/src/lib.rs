pub use type_id;
pub use type_id::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FuncOutput {
    Unary(Ty),
    ServerStream { yield_ty: Ty, return_ty: Ty },
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Func {
    pub docs: String,
    pub index: u16,
    pub ident: Ident,
    pub args: Vec<Ty>,
    pub output: FuncOutput,
}

#[derive(Default)]
#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeDef {
    pub name: Ident,
    pub docs: String,
    pub costom_types: CostomTypes,
    pub funcs: Vec<Func>,
}

impl TypeDef {
    pub fn new(name: &str, costom_types: CostomTypes, funcs: Vec<Func>, docs: &str) -> Self {
        Self {
            name: Ident(name.to_string()),
            costom_types,
            funcs,
            docs: docs.to_string(),
        }
    }
}
