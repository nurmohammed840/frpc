use super::*;
use crate::utils::write_doc_comments;

pub fn main(f: &mut impl Write, provider: &CodeGen) -> Result {
    writeln!(f, "let struct = {{")?;
    for path in &provider.output_paths {
        let ident = object_ident_from(path);
        writeln!(f, "{ident}(d: use.Decoder): {ident} {{")?;

        match &provider.type_def.costom_types[*path] {
            CustomTypeKind::Unit(data) => {
                let items = Fmt(|f| {
                    data.fields
                        .iter()
                        .enumerate()
                        .try_for_each(|(i, UnitField { name, .. })| {
                            writeln!(f, "case {i}: return {ident}.{name};")
                        })
                });
                write_enum(f, &ident, items)?;
            }
            CustomTypeKind::Enum(data) => {
                let items = Fmt(|f| {
                    data.fields.iter().enumerate().try_for_each(
                        |(i, EnumField { name, kind, .. })| {
                            writeln!(f, "case {i}: return {{\ntype: {name:?},")?;
                            match kind {
                                EnumKind::Struct(fields) => write_struct(f, fields)?,
                                EnumKind::Tuple(fields) => {
                                    for (i, TupleField { doc, ty }) in fields.iter().enumerate() {
                                        write_doc_comments(f, doc)?;
                                        writeln!(f, " {i}: {}(),", fmt_ty(ty, "struct"))?;
                                    }
                                }
                                EnumKind::Unit => {}
                            }
                            writeln!(f, "}};")
                        },
                    )
                });
                write_enum(f, &ident, items)?;
            }
            CustomTypeKind::Struct(data) => {
                writeln!(f, "return {{")?;
                write_struct(f, &data.fields)?;
                writeln!(f, "}}")?
            }
            CustomTypeKind::Tuple(data) => {
                writeln!(f, "return {}();", fmt_tuple(&data.fields, "struct"))?;
            }
        }
        writeln!(f, "}},")?;
    }
    writeln!(f, "}}")
}

fn write_struct(f: &mut impl Write, fields: &[StructField]) -> Result {
    fields.iter().try_for_each(|StructField { doc, name, ty }| {
        write_doc_comments(f, doc)?;
        writeln!(f, "{name}: {}(),", fmt_ty(ty, "struct"))
    })
}

fn write_enum(f: &mut impl Write, ident: &String, items: fmt!(type)) -> Result {
    writeln!(f, "const num = d.len_u15();\nswitch (num) {{\n{items}default: throw use.enumErr({ident:?}, num);\n}}")
}
