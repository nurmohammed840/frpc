use super::*;

pub fn main(f: &mut impl Write, provider: &CodeGen) -> Result {
    writeln!(f, "let extern = {{")?;

    for path in &provider.input_paths {
        let ident = object_ident_from(path);
        writeln!(f, "{ident}(d: use.BufWriter, z: {ident}) {{")?;

        match &provider.type_def.costom_types[*path] {
            CustomTypeKind::Unit(data) => {
                writeln!(f, "switch (z) {{")?;
                for (i, UnitField { name, .. }) in data.fields.iter().enumerate() {
                    writeln!(f, "case {ident}.{name}: return d.len_u15({i});")?;
                }
                writeln!(f, "}}")?;
            }
            CustomTypeKind::Enum(data) => {
                writeln!(f, "switch (z.type) {{")?;
                for (i, EnumField { name, kind, .. }) in data.fields.iter().enumerate() {
                    writeln!(f, "case {name:?}: d.len_u15({i});")?;
                    match kind {
                        EnumKind::Struct(fields) => write_struct(f, fields)?,
                        EnumKind::Tuple(fields) => {
                            for (i, TupleField { ty, .. }) in fields.iter().enumerate() {
                                writeln!(f, "{}(z[{i}]);", fmt_ty(ty, "extern"))?;
                            }
                        }
                        EnumKind::Unit => {}
                    }
                    writeln!(f, "break;")?;
                }
                writeln!(f, "}}")?;
            }
            CustomTypeKind::Struct(data) => write_struct(f, &data.fields)?,
            CustomTypeKind::Tuple(data) => {
                writeln!(f, "return {}(z);", fmt_tuple(&data.fields, "extern"))?;
            }
        }
        writeln!(f, "}},")?;
    }
    writeln!(f, "}}")
}

fn write_struct(f: &mut impl Write, fields: &[StructField]) -> Result {
    fields.iter().try_for_each(|StructField { name, ty, .. }| {
        writeln!(f, "{}(z.{name});", fmt_ty(ty, "extern"))
    })
}
