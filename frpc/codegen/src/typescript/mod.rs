use std::collections::btree_map::BTreeMap;

pub mod generate;
pub mod interface;
use crate::CodeGen;
use crate::{fmt, utils::uppercase_first, Fmt};

impl CodeGen<'_> {
    pub fn typescript(&self) -> fmt!(type '_) {
        Fmt(move |f| {
            let ident_map = IdentMap::new(self.type_def.costom_types.keys().map(|k| k.as_str()));
            // TODO: make a struct that hold `f` and `ident_map`, and pass that into function.
            // instead of passing both (f, ident_map)

            for (path, value) in self.type_def.costom_types.iter() {
                interface::gen_type(f, &ident_map, path, value)?;
            }
            generate::decoder::main(f, self, &ident_map)?;
            generate::encoder::main(f, self, &ident_map)?;
            generate::stub::main(f, &self.type_def, &ident_map)
        })
    }
}

pub struct IdentMap<'a>(pub BTreeMap<&'a str, String>);

impl<'a> IdentMap<'a> {
    fn new(paths: impl IntoIterator<Item = &'a str>) -> Self {
        let mut list = paths
            .into_iter()
            .map(|path_str| {
                let path = path_str
                    .split("::")
                    .map(|entry| entry.strip_prefix("r#").unwrap_or(entry).to_owned())
                    .collect::<Vec<_>>();

                (path, path_str)
            })
            .collect::<Vec<_>>();

        list.sort_by(|(path_a, ..), (path_b, ..)| path_a.len().cmp(&path_b.len()));

        let mut ident_map = BTreeMap::new();

        for (mut path, path_str) in list {
            let mut entry = uppercase_first(&path.pop().unwrap());
            while ident_map.contains_key(&*entry) {
                let last = uppercase_first(&path.pop().unwrap());
                entry = format!("{last}_{entry}");
            }
            ident_map.insert(path_str, entry);
        }
        Self(ident_map)
    }
}

impl<'a> std::ops::Deref for IdentMap<'a> {
    type Target = BTreeMap<&'a str, String>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
