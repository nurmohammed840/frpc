use crate::typescript::interface::EnumReprValue;

use super::*;

pub fn main(f: &mut impl Write, provider: &CodeGen, ident_map: &IdentMap) -> Result {
    writeln!(f, "let extern = {{")?;

    for path in &provider.input_paths {
        let ident = &ident_map[path];
        writeln!(f, "{ident}(d: use.BufWriter, z: {ident}) {{")?;

        match &provider.type_def.costom_types[*path] {
            CustomTypeKind::Unit(data) => {
                writeln!(f, "switch (z) {{")?;
                for UnitField { name, value, .. } in data.fields.iter() {
                    let index = EnumReprValue(*value);
                    let repr_ty = enum_repr_ty(value);
                    writeln!(f, "case {ident}.{name}: return d.{repr_ty}({index});")?;
                }
                writeln!(f, "}}")?;
            }
            CustomTypeKind::Enum(data) => {
                let mut i = EnumFieldIndex(0);
                writeln!(f, "switch (z.type) {{")?;
                for EnumField {
                    name, kind, index, ..
                } in data.fields.iter()
                {
                    let repr_ty = match index {
                        Some(value) => enum_repr_ty(value).to_string(),
                        None => String::from("len_u15"),
                    };
                    let index = i.get(index);
                    writeln!(f, "case {name:?}: d.{repr_ty}({index});")?;
                    match kind {
                        EnumKind::Struct(fields) => write_struct(f, fields, ident_map)?,
                        EnumKind::Tuple(fields) => {
                            for (i, TupleField { ty, .. }) in fields.iter().enumerate() {
                                writeln!(f, "{}(z[{i}]);", fmt_ty(ty, "extern", ident_map))?;
                            }
                        }
                        EnumKind::Unit => {}
                    }
                    writeln!(f, "break;")?;
                }
                writeln!(f, "}}")?;
            }
            CustomTypeKind::Struct(data) => write_struct(f, &data.fields, ident_map)?,
            CustomTypeKind::Tuple(data) => {
                writeln!(
                    f,
                    "return {}(z);",
                    fmt_tuple(&data.fields, "extern", ident_map)
                )?;
            }
        }
        writeln!(f, "}},")?;
    }
    writeln!(f, "}}")
}

fn write_struct(f: &mut impl Write, fields: &[StructField], ident_map: &IdentMap) -> Result {
    fields.iter().try_for_each(|StructField { name, ty, .. }| {
        writeln!(f, "{}(z.{name});", fmt_ty(ty, "extern", ident_map))
    })
}
