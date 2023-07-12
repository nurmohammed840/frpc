use super::*;
use std::{collections::*, hash::Hash};

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SetVariant {
    BTreeSet,
    HashSet,
    BinaryHeap,
    LinkedList,
    VecDeque,
    Vec,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "hash", derive(Hash))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MapVariant {
    HashMap,
    BTreeMap,
}

macro_rules! impl_ty_class {
    [Set for $name: tt <$($ty_arg: ty),*> where $($ty: tt)*] => {
        impl<$($ty)*> TypeId for $name<$($ty_arg),*> {
            fn ty(costom_types: &mut CostomTypes) -> Ty {
                Ty::Set {
                    variant: SetVariant::$name,
                    ty: Box::new(T::ty(costom_types)),
                }
            }
        }
    };
    [Map for $name: tt <$($ty_arg: ty),*> where $($ty: tt)*] => {
        impl<$($ty)*> TypeId for $name<$($ty_arg),*> {
            fn ty(costom_types: &mut CostomTypes) -> Ty {
                Ty::Map {
                    variant: MapVariant::$name,
                    ty: Box::new((K::ty(costom_types), V::ty(costom_types))),
                }
            }
        }
    };
}

impl_ty_class!(Set for Vec<T>             where T: TypeId);
impl_ty_class!(Set for VecDeque<T>        where T: TypeId);
impl_ty_class!(Set for LinkedList<T>      where T: TypeId);
impl_ty_class!(Set for BTreeSet<T>        where T: TypeId + Ord);
impl_ty_class!(Set for BinaryHeap<T>      where T: TypeId + Ord);
impl_ty_class!(Set for HashSet<T>         where T: TypeId + Eq + Hash);
impl_ty_class!(Map for BTreeMap<K, V>     where K: TypeId + Ord, V: TypeId);
impl_ty_class!(Map for HashMap<K, V>      where K: TypeId + Eq + Hash, V: TypeId);

impl<T: TypeId> TypeId for &[T] {
    fn ty(costom_types: &mut CostomTypes) -> Ty {
        Ty::Set {
            variant: SetVariant::Vec,
            ty: Box::new(T::ty(costom_types)),
        }
    }
}
