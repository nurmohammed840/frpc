use super::*;
use crate::{typescript::interface::EnumReprValue, utils::write_doc_comments};

pub fn main(f: &mut impl Write, provider: &CodeGen, ident_map: &IdentMap) -> Result {
    writeln!(f, "let struct = {{")?;
    for path in &provider.output_paths {
        let ident = &ident_map[path];
        writeln!(f, "{ident}(d: use.Decoder): {ident} {{")?;

        match &provider.type_def.costom_types[*path] {
            CustomTypeKind::Unit(data) => {
                let items = Fmt(|f| {
                    data.fields
                        .iter()
                        .try_for_each(|UnitField { name, value, .. }| {
                            let index = EnumReprValue(*value);
                            writeln!(f, "case {index}: return {ident}.{name};")
                        })
                });
                let repr_ty = enum_repr_ty(data.enum_repr());
                writeln!(f, "const num = d.{repr_ty}();")?;
                write_enum(f, &ident, items)?;
            }
            CustomTypeKind::Enum(data) => {
                let items = Fmt(|f| {
                    let mut i = EnumFieldIndex(0);
                    data.fields.iter().try_for_each(
                        |EnumField {
                             name, kind, index, ..
                         }| {
                            let index = i.get(index);
                            writeln!(f, "case {index}: return {{\ntype: {name:?},")?;
                            match kind {
                                EnumKind::Struct(fields) => write_struct(f, fields, ident_map)?,
                                EnumKind::Tuple(fields) => {
                                    for (i, TupleField { doc, ty }) in fields.iter().enumerate() {
                                        write_doc_comments(f, doc)?;
                                        writeln!(f, " {i}: {}(),", fmt_ty(ty, "struct", ident_map))?;
                                    }
                                }
                                EnumKind::Unit => {}
                            }
                            writeln!(f, "}};")
                        },
                    )
                });
                match data.enum_repr() {
                    None => f.write_str("const num = d.len_u15();\n"),
                    Some(repr) => writeln!(f, "const num = d.{}();", enum_repr_ty(repr)),
                }?;
                write_enum(f, &ident, items)?;
            }
            CustomTypeKind::Struct(data) => {
                f.write_str("return {\n")?;
                write_struct(f, &data.fields, ident_map)?;
                f.write_str("}\n")?;
            }
            CustomTypeKind::Tuple(data) => {
                writeln!(f, "return {}();", fmt_tuple(&data.fields, "struct", ident_map))?;
            }
        }
        writeln!(f, "}},")?;
    }
    writeln!(f, "}}")
}

fn write_struct(f: &mut impl Write, fields: &[StructField], ident_map: &IdentMap) -> Result {
    fields.iter().try_for_each(|StructField { doc, name, ty }| {
        write_doc_comments(f, doc)?;
        writeln!(f, "{name}: {}(),", fmt_ty(ty, "struct", ident_map))
    })
}

fn write_enum(f: &mut impl Write, ident: &str, items: fmt!(type)) -> Result {
    writeln!(
        f,
        "switch (num) {{\n{items}default: throw use.enumErr({ident:?}, num);\n}}"
    )
}
