mod basic;
mod collection;
mod wrapper;

use std::collections::btree_map;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use collection::{MapVariant, SetVariant};

pub trait TypeId {
    fn ty(_: &mut CostomTypes) -> Ty;
}

#[allow(non_camel_case_types)]
#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Ty {
    // Never, // debug
    u8,
    u16,
    u32,
    u64,
    u128,

    i8,
    i16,
    i32,
    i64,
    i128,

    f32,
    f64,

    bool,

    // char,
    String,

    Option(Box<Ty>),
    Result(Box<(Ty, Ty)>),

    Tuple(Vec<Ty>),

    Array {
        ty: Box<Ty>,
        len: usize,
    },
    Set {
        variant: SetVariant,
        ty: Box<Ty>,
    },
    Map {
        variant: MapVariant,
        ty: Box<(Ty, Ty)>,
    },

    /// The path of the user-defined type
    ///
    /// ```
    ///    struct Bar { ... }  enum Foo { ... }
    /// //        ^^^               ^^^
    /// //           \             /
    /// //    Type::CustomType("<path>::Bar" | "<path>::Foo")
    /// ```
    CustomType(String),
}

impl Ty {
    pub fn is_empty_tuple(&self) -> bool {
        match self {
            Ty::Tuple(tys) => tys.is_empty(),
            _ => false,
        }
    }
}

#[derive(Default)]
#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CostomTypes(BTreeMap<String, CustomTypeKind>);

impl CostomTypes {
    pub fn register(&mut self, name: String, f: fn(&mut Self) -> CustomTypeKind) -> Ty {
        if let btree_map::Entry::Vacant(entry) = self.0.entry(name.clone()) {
            entry.insert(CustomTypeKind::default());
            let costom_type_kind = f(self);
            self.0.insert(name.clone(), costom_type_kind);
        }
        Ty::CustomType(name)
    }
}

impl Deref for CostomTypes {
    type Target = BTreeMap<String, CustomTypeKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for CustomTypeKind {
    fn default() -> Self {
        Self::Unit(CustomType {
            doc: "".into(),
            fields: vec![],
        })
    }
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CustomTypeKind {
    Unit(CustomType<UnitField>),
    Enum(CustomType<EnumField>),
    Tuple(CustomType<TupleField>),
    Struct(CustomType<StructField>),
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Any user defined type like: `struct`, `enum`
pub struct CustomType<Field> {
    pub doc: String,
    pub fields: Vec<Field>,
}

impl CustomType<UnitField> {
    pub fn enum_repr(&self) -> &EnumRepr {
        &self.fields.first().unwrap().value
    }
}

impl CustomType<EnumField> {
    pub fn enum_repr(&self) -> Option<&EnumRepr> {
        self.fields.first().unwrap().index.as_ref()
    }
}

macro_rules! enum_repr {
    [$($ty: tt)*] => {
        #[cfg_attr(feature = "hash", derive(Hash))]
        #[cfg_attr(feature = "debug", derive(Debug))]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        pub enum EnumRepr { $($ty($ty)),* }
        $(
            impl From<$ty> for EnumRepr {
                fn from(v: $ty) -> Self {
                    Self::$ty(v)
                }
            }
        )*
        impl fmt::Display for EnumRepr {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$ty(v) => v.fmt(f)),*
                }
            }
        }
    };
}

enum_repr! {
    u8
    u16
    u32
    u64
    usize
    i8
    i16
    i32
    i64
    isize
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnitField {
    pub doc: String,
    pub name: Ident,
    pub value: EnumRepr,
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumField {
    pub doc: String,
    pub name: Ident,
    pub kind: EnumKind,
    pub index: Option<EnumRepr>,
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EnumKind {
    Unit,
    Struct(Vec<StructField>),
    Tuple(Vec<TupleField>),
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructField {
    pub doc: String,
    pub name: Ident,
    pub ty: Ty,
}

#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TupleField {
    pub doc: String,
    pub ty: Ty,
}

// ---------------------------------------------------------------

#[derive(Default, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ident(pub String);

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.trim_start_matches("r#")
    }
}

impl Ident {
    pub fn is_raw_str_literal(&self) -> bool {
        self.0.starts_with("r#")
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.trim_start_matches("r#").fmt(f)
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.trim_start_matches("r#").fmt(f)
    }
}

// ---------------------------------------------------------------

impl<Field> CustomType<Field> {
    pub fn new(doc: &str, fields: Vec<Field>) -> Self {
        Self {
            doc: doc.to_string(),
            fields,
        }
    }
}

impl UnitField {
    pub fn new(doc: &str, name: &str, value: EnumRepr) -> Self {
        Self {
            doc: doc.to_string(),
            name: Ident(name.to_string()),
            value,
        }
    }
}

impl EnumField {
    pub fn new(doc: &str, name: &str, index: Option<EnumRepr>, kind: EnumKind) -> Self {
        Self {
            doc: doc.to_string(),
            name: Ident(name.to_string()),
            kind,
            index,
        }
    }
}

impl StructField {
    pub fn new(doc: &str, name: &str, ty: Ty) -> Self {
        Self {
            doc: doc.to_string(),
            name: Ident(name.to_string()),
            ty,
        }
    }
}

impl TupleField {
    pub fn new(doc: &str, ty: Ty) -> Self {
        Self {
            doc: doc.to_string(),
            ty,
        }
    }
}
