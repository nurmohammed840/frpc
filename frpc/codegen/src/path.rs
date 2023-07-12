use frpc_message::*;

pub struct Path<'a> {
    costom_types: &'a CostomTypes,
    pub paths: Vec<&'a str>,
}

impl<'a> Path<'a> {
    pub fn new(costom_types: &'a CostomTypes) -> Self {
        Self {
            paths: vec![],
            costom_types,
        }
    }
}

impl<'a> Path<'a> {
    pub fn add_tys(&mut self, tys: impl Iterator<Item = &'a Ty>) {
        for ty in tys {
            self.add_ty(ty)
        }
    }

    pub fn add_ty(&mut self, ty: &'a Ty) {
        match ty {
            Ty::Map { ty, .. } => self.add_ty(&ty.1),
            Ty::Result(ty) => {
                self.add_ty(&ty.0);
                self.add_ty(&ty.1);
            }
            Ty::Tuple(tys) => self.add_tys(tys.iter()),
            Ty::Option(ty) | Ty::Array { ty, .. } | Ty::Set { ty, .. } => self.add_ty(ty),
            Ty::CustomType(path) if !self.paths.contains(&path.as_str()) => {
                self.paths.push(path);
                match &self.costom_types[path] {
                    CustomTypeKind::Enum(data) => {
                        for data in data.fields.iter() {
                            match &data.kind {
                                EnumKind::Tuple(fields) => {
                                    self.add_tys(fields.iter().map(|f| &f.ty))
                                }
                                EnumKind::Struct(fields) => {
                                    self.add_tys(fields.iter().map(|f| &f.ty))
                                }
                                EnumKind::Unit => {}
                            }
                        }
                    }
                    CustomTypeKind::Tuple(data) => self.add_tys(data.fields.iter().map(|f| &f.ty)),
                    CustomTypeKind::Struct(data) => self.add_tys(data.fields.iter().map(|f| &f.ty)),
                    CustomTypeKind::Unit(_) => {}
                }
            }
            _ => {}
        }
    }
}
