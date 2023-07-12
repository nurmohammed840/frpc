pub mod generate;
pub mod interface;
use crate::CodeGen;
use crate::{fmt, utils::object_ident_from, Fmt};

impl CodeGen<'_> {
    pub fn typescript(&self) -> fmt!(type '_) {
        Fmt(move |f| {
            for (path, value) in self.type_def.costom_types.iter() {
                interface::gen_type(f, object_ident_from(path), value)?;
            }
            generate::decoder::main(f, self)?;
            generate::encoder::main(f, self)?;
            generate::stub::main(f, &self.type_def)
        })
    }
}
