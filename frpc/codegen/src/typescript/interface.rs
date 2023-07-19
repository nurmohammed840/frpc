use crate::utils::{join, object_ident_from, write_doc_comments};
use frpc_message::*;
use std::fmt::{Display, Result, Write};

pub(super) struct EnumReprValue(pub EnumRepr);

impl Display for EnumReprValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        match self.0 {
            EnumRepr::u64(v) => write!(f, "{v}n"),
            EnumRepr::i64(v) => write!(f, "{v}n"),
            EnumRepr::usize(v) if usize::BITS >= 64 => write!(f, "{v}n"),
            EnumRepr::isize(v) if isize::BITS >= 64 => write!(f, "{v}n"),
            num => num.fmt(f),
        }
    }
}

pub fn gen_type(f: &mut impl Write, ident: String, kind: &CustomTypeKind) -> Result {
    match kind {
        CustomTypeKind::Unit(unit) => {
            write_doc_comments(f, &unit.doc)?;

            write!(f, "export const {ident} = ")?;
            write_map(
                f,
                ":",
                unit.fields
                    .iter()
                    .map(|f| (&f.doc, &f.name, EnumReprValue(f.value))),
            )?;
            let enum_type = match unit.fields.first().unwrap().value {
                EnumRepr::u8(_)
                | EnumRepr::u16(_)
                | EnumRepr::u32(_)
                | EnumRepr::i8(_)
                | EnumRepr::i16(_)
                | EnumRepr::i32(_) => "number",

                EnumRepr::isize(_) | EnumRepr::usize(_) if usize::BITS <= 32 => "number",
                _ => "bigint",
            };
            writeln!(f, "export type {ident} = {enum_type};")?;
        }
        CustomTypeKind::Struct(data) => {
            write_doc_comments(f, &data.doc)?;

            write!(f, "export interface {ident} ")?;
            let fields = data
                .fields
                .iter()
                .map(|f| (&f.doc, &f.name, fmt_js_ty(&f.ty)));

            write_map(f, ":", fields)?;
        }
        CustomTypeKind::Tuple(data) => {
            write_doc_comments(f, &data.doc)?;
            let fields = join(data.fields.iter().map(|f| fmt_js_ty(&f.ty)), ", ");
            writeln!(f, "export type {ident} = [{fields}];")?;
        }
        CustomTypeKind::Enum(data) => {
            write_doc_comments(f, &data.doc)?;

            writeln!(f, "export type {ident} =")?;

            for EnumField {
                doc: _, name, kind, ..
            } in &data.fields
            {
                let fields = match kind {
                    EnumKind::Unit => String::new(),
                    EnumKind::Struct(dta) => join(
                        dta.iter()
                            .map(|f| format!("{}: {}", f.name, fmt_js_ty(&f.ty))),
                        ", ",
                    ),
                    EnumKind::Tuple(data) => join(
                        data.iter()
                            .enumerate()
                            .map(|(i, field)| format!("{i}: {}", fmt_js_ty(&field.ty))),
                        ", ",
                    ),
                };
                writeln!(f, "| {{ type: {name:?}, {fields}}}")?;
            }
        }
    }
    Ok(())
}

fn write_map<'a, I, K, V>(f: &mut impl Write, sep: &str, fields: I) -> Result
where
    K: Display,
    V: Display,
    I: Iterator<Item = (&'a String, K, V)>,
{
    writeln!(f, "{{")?;
    for (doc, name, item) in fields {
        write_doc_comments(f, doc)?;
        writeln!(f, "{name}{sep} {item},")?;
    }
    writeln!(f, "}}")
}

pub fn fmt_js_ty(ty: &Ty) -> String {
    match ty {
        Ty::u8 | Ty::u16 | Ty::u32 | Ty::i8 | Ty::i16 | Ty::i32 | Ty::f32 | Ty::f64 => {
            "number".into()
        }
        Ty::u64 | Ty::i64 | Ty::u128 | Ty::i128 => "bigint".into(),
        Ty::bool => "boolean".into(),
        // Ty::char |
        Ty::String => "string".into(),

        Ty::Array { ty, .. } | Ty::Set { ty, .. } => match **ty {
            Ty::u8 => "Uint8Array",
            Ty::i8 => "Int8Array",
            Ty::f32 => "Float32Array",
            Ty::f64 => "Float64Array",
            _ => return format!("Array<{}>", fmt_js_ty(ty)),
        }
        .to_string(),

        Ty::Option(ty) => format!("use.Option<{}>", fmt_js_ty(ty)),
        Ty::Result(ty) => format!("use.Result<{}, {}>", fmt_js_ty(&ty.0), fmt_js_ty(&ty.1)),

        Ty::Map { ty, .. } => format!("Map<{}, {}>", fmt_js_ty(&ty.0), fmt_js_ty(&ty.1)),
        Ty::Tuple(tys) => {
            if tys.is_empty() {
                "null".to_string()
            } else {
                format!("[{}]", join(tys.iter().map(fmt_js_ty), ", "))
            }
        }
        Ty::CustomType(path) => object_ident_from(path),
    }
}
