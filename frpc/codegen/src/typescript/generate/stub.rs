use super::*;
use crate::{typescript::interface::fmt_js_ty, utils::write_doc_comments};

pub fn main(f: &mut impl Write, type_def: &TypeDef, ident_map: &IdentMap) -> Result {
    write_doc_comments(f, &type_def.docs)?;
    writeln!(f, "export default class Self {{")?;
    writeln!(f, "constructor(private rpc: use.RpcTransport) {{}}")?;
    writeln!(f, "static close(this: Self) {{ this.rpc.close() }}")?;

    type_def.funcs.iter().try_for_each(
        |Func {
             index,
             ident: path,
             args,
             output,
             docs,
         }| {
            let ident = path.replace("::", "_");

            write_doc_comments(f, docs)?;
            write!(f, "{ident}(")?;
            for (num, ty) in args.iter().enumerate() {
                write!(f, "_{num}: {}, ", fmt_js_ty(ty, &ident_map))?;
            }
            writeln!(f, ") {{")?;
            {
                let rpc_type = match output {
                    FuncOutput::Unary(_) => "unary",
                    FuncOutput::ServerStream { .. } => "sse",
                };
                writeln!(f, "return (requestInit: RequestInit = {{}}) => use.make_call(this.rpc, {rpc_type:?}, {index}, requestInit,")?;
                writeln!(f, "d => {{")?;
                for (num, arg) in args.iter().enumerate() {
                    match arg {
                        Ty::CustomType(path) => {
                            writeln!(f, "extern.{}(d, _{num});", ident_map[path.as_str()])?
                        }
                        ty => writeln!(f, "{}(_{num});", fmt_ty(ty, "extern", ident_map))?,
                    };
                }
                writeln!(f, "}},")?;

                match output {
                    FuncOutput::Unary(retn) => {
                        writeln!(f, "async data => {{")?;
                        writeln!(f, "let _buf = await data")?;
                        if !retn.is_empty_tuple() {
                            writeln!(f, "let d = use.Decoder.from(_buf);")?;
                            writeln!(f, "return {}", decode_data(retn, ident_map))?;
                        }
                        writeln!(f, "}},")?;
                    }
                    FuncOutput::ServerStream {
                        return_ty,
                        yield_ty,
                    } => {
                        writeln!(f, "async function* (s) {{")?;
                        writeln!(f, "while (true) {{")?;
                        writeln!(f, "let {{ value, done }} = await s.next();")?;
                        writeln!(f, "let d = use.Decoder.from(value);")?;
                        writeln!(f, "if (done) {{")?;
                        writeln!(f, "return {}", decode_data(return_ty, ident_map))?;
                        writeln!(f, "}}")?;
                        writeln!(f, "yield {}", decode_data(yield_ty, ident_map))?;
                        writeln!(f, "}}")?;
                        writeln!(f, "}}")?;
                    }
                };
                writeln!(f, ")")?;
            }
            writeln!(f, "}}")
        },
    )?;
    writeln!(f, "}}")
}

fn decode_data(ty: &Ty, ident_map: &IdentMap) -> String {
    if ty.is_empty_tuple() {
        return String::new();
    }
    match ty {
        Ty::CustomType(path) => {
            format!("struct.{}(d)", ident_map[path.as_str()])
        }
        ty => format!("{}()", fmt_ty(ty, "struct", ident_map)),
    }
}
