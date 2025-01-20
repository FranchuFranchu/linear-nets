use super::{system::Cell, Net};
pub fn apply_rule(mut net: &mut Net, left: Cell, right: Cell) {
    if is_defined(&left, &right) {
        apply_rule_inner(&mut net, left, right);
    } else if is_defined(&right, &left) {
        apply_rule_inner(&mut net, right, left);
    } else {
        todo!();
    }
}
pub fn is_defined(left: &Cell, right: &Cell) -> bool {
    use Cell::*;
    match (left, right) {
        (Times(..), Par(..)) | (One(..), False(..)) => true,
        _ => false,
    }
}
pub fn apply_rule_inner(net: &mut Net, left: Cell, right: Cell) {
    use Cell::*;
    match (left, right) {
        (Times((a,), (b,)), Par((c, d))) => {
            // Annihilate
            net.link(a, c);
            net.link(b, d);
        }
        (One(), False((a,), b)) => {
            net.plug_box(b, vec![a]);
        }
        _ => {}
    }
}
