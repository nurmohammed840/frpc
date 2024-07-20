use super::IdentMap;
use crate::CodeGen;
pub mod decoder;
pub mod encoder;
pub mod stub;

use crate::{fmt, Fmt};
use frpc_message::*;
use std::fmt::{Result, Write};

fn fmt_tuple<'a>(
    fields: &'a [TupleField],
    scope: &'static str,
    ident_map: &'a IdentMap,
) -> fmt!(type 'a) {
    Fmt(move |f| {
        write!(f, "d.tuple(")?;
        for TupleField { ty, .. } in fields.iter() {
            write!(f, "{},", fmt_ty(ty, scope, ident_map))?;
        }
        write!(f, ")")
    })
}

fn fmt_ty<'a>(ty: &'a Ty, scope: &'a str, ident_map: &'a IdentMap) -> fmt!(type 'a) {
    Fmt(move |f| match ty {
        Ty::u8 => write!(f, "d.u8"),
        Ty::u16 => write!(f, "d.num('U', 16)"),
        Ty::u32 => write!(f, "d.num('U', 32)"),
        Ty::u64 => write!(f, "d.num('U', 64)"),
        Ty::u128 => write!(f, "d.num('U', 128)"),

        Ty::i8 => write!(f, "d.i8"),
        Ty::i16 => write!(f, "d.num('I', 16)"),
        Ty::i32 => write!(f, "d.num('I', 32)"),
        Ty::i64 => write!(f, "d.num('I', 64)"),
        Ty::i128 => write!(f, "d.num('I', 128)"),

        Ty::f32 => write!(f, "d.f32"),
        Ty::f64 => write!(f, "d.f64"),

        Ty::bool => write!(f, "d.bool"),

        // Ty::char => write!(f, "d.char"),
        Ty::String => write!(f, "d.str"),

        Ty::Option(ty) => write!(f, "d.option({})", fmt_ty(ty, scope, ident_map)),
        Ty::Result(ty) => write!(
            f,
            "d.result({}, {})",
            fmt_ty(&ty.0, scope, ident_map),
            fmt_ty(&ty.1, scope, ident_map)
        ),

        Ty::Tuple(tys) => {
            if tys.is_empty() {
                write!(f, "d.null")
            } else {
                write!(f, "d.tuple(")?;
                tys.iter()
                    .try_for_each(|ty| write!(f, "{},", fmt_ty(ty, scope, ident_map)))?;
                write!(f, ")")
            }
        }
        Ty::Array { len, ty } => match ty.as_ref() {
            Ty::u8 => write!(f, "d.fixed_buf('u8', {len})"),
            Ty::i8 => write!(f, "d.fixed_buf('i8', {len})"),
            Ty::f32 => write!(f, "d.fixed_buf('f32', {len})"),
            Ty::f64 => write!(f, "d.fixed_buf('f64', {len})"),
            ty => write!(f, "d.fixed_arr({}, {len})", fmt_ty(ty, scope, ident_map)),
        },
        Ty::Set { ty, .. } => match ty.as_ref() {
            Ty::u8 => write!(f, "d.buf('u8')"),
            Ty::i8 => write!(f, "d.buf('i8')"),
            Ty::f32 => write!(f, "d.buf('f32')"),
            Ty::f64 => write!(f, "d.buf('f64')"),
            ty => write!(f, "d.arr({})", fmt_ty(ty, scope, ident_map)),
        },
        Ty::Map { ty, .. } => write!(
            f,
            "d.map({}, {})",
            fmt_ty(&ty.0, scope, ident_map),
            fmt_ty(&ty.1, scope, ident_map)
        ),
        Ty::CustomType(path) => write!(f, "{scope}.{}.bind(0, d)", ident_map[path.as_str()]),
    })
}

struct EnumFieldIndex(pub u32);

impl EnumFieldIndex {
    fn get(&mut self, index: &Option<EnumRepr>) -> String {
        match index {
            Some(value) => super::interface::EnumReprValue(*value).to_string(),
            None => {
                let index = self.0.to_string();
                self.0 += 1;
                index
            }
        }
    }
}

fn enum_repr_ty(enum_repr: &EnumRepr) -> fmt!(type '_) {
    Fmt(move |f| match enum_repr {
        EnumRepr::u8(_) => f.write_str("u8"),
        EnumRepr::i8(_) => f.write_str("i8"),
        EnumRepr::u16(_) => f.write_str("num('U', 16)"),
        EnumRepr::u32(_) => f.write_str("num('U', 32)"),
        EnumRepr::u64(_) => f.write_str("num('U', 64)"),
        EnumRepr::i16(_) => f.write_str("num('I', 16)"),
        EnumRepr::i32(_) => f.write_str("num('I', 32)"),
        EnumRepr::i64(_) => f.write_str("num('I', 64)"),
        EnumRepr::usize(_) => write!(f, "num('U', {})", usize::BITS),
        EnumRepr::isize(_) => write!(f, "num('I', {})", isize::BITS),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_fmt_tuple() {
        use Ty::*;
        let tys = vec![
            Option(Box::new(bool)),
            Result(Box::new((CustomType("::path::ident".into()), String))),
            Map {
                variant: MapVariant::BTreeMap,
                ty: Box::new((String, Set { variant: SetVariant::BTreeSet, ty: Box::new(u8), })),
            },
        ];
        assert_eq!(
            format!("{}", fmt_ty(&Tuple(tys), "This", &IdentMap::new(["::path::ident"]))),
            "d.tuple(d.option(d.bool),d.result(This.PathIdent.bind(0, d), d.str),d.map(d.str, d.vec(d.u8)),)"
        );
    }
}
