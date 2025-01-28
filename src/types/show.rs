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
            Type::Ofc(t) => format!("!{}", t.show(scope)),
            Type::Why(t) => format!("?{}", t.show(scope)),
            Type::All(id, body) => format!("∀{}.{}", pick_name(scope, *id), body.show(scope)),
            Type::Any(id, body) => format!("∃{}.{}", pick_name(scope, *id), body.show(scope)),
            Type::Var(id, invert) | Type::Eigenvar(id, invert) => {
                pick_name(scope, *id) + if *invert { "'" } else { "" }
            }
            Type::Hole => format!("_"),
            Type::Error => format!("Error"),
            x => todo!("Can not show {:?}", x),
        }
    }
}
