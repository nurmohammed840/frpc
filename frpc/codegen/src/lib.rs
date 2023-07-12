use std_lib::fmt::{fmt, Fmt};

mod path;
pub mod typescript;
pub mod utils;

use frpc_message::{FuncOutput, TypeDef};
use path::Path;

pub struct CodeGen<'a> {
    pub type_def: &'a TypeDef,
    input_paths: Vec<&'a str>,
    output_paths: Vec<&'a str>,
}

impl<'a> From<&'a TypeDef> for CodeGen<'a> {
    fn from(type_def: &'a TypeDef) -> Self {
        let mut input = Path::new(&type_def.costom_types);
        let mut output = Path::new(&type_def.costom_types);

        input.add_tys(type_def.funcs.iter().flat_map(|func| func.args.iter()));
        output.add_tys(type_def.funcs.iter().flat_map(|func| match &func.output {
            FuncOutput::Unary(ty) => vec![ty],
            FuncOutput::ServerStream {
                yield_ty,
                return_ty,
            } => vec![yield_ty, return_ty],
        }));

        let input_paths = input.paths;
        let output_paths = output.paths;

        Self {
            type_def,
            input_paths,
            output_paths,
        }
    }
}
