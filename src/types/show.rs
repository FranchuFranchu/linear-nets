use crate::util::pick_name;

use crate::types::Type;

use crate::net::VarId;

use crate::types::BTreeMap;

impl Type {
    pub fn show(&self, scope: &mut BTreeMap<VarId, String>) -> String {
        match self {
            Type::Times(a, b) => {
                format!("({} ⊗ {})", a.show(scope), b.show(scope))
            }
            Type::One => format!("1"),
            Type::Par(a, b) => {
                format!("({} ⅋ {})", a.show(scope), b.show(scope))
            }
            Type::False => format!("⊥"),
            Type::Plus(a, b) => {
                format!("({} ⊕ {})", a.show(scope), b.show(scope))
            }
            Type::With(a, b) => {
                format!("({} & {})", a.show(scope), b.show(scope))
            }
            Type::Var(id, invert) => pick_name(scope, *id) + if *invert { "'" } else { "" },
            x => todo!("Can not show {:?}", x),
        }
    }
}
