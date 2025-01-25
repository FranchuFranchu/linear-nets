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
        (Times(..), Par(..))
        | (One(..), False(..))
        | (Left(..), With(..))
        | (Right(..), With(..))
        | (Exp0(..), Weak(..))
        | (Exp0(..), Dere(..))
        | (Exp0(..), Cntr(..))
        | (Exp1(..), Weak(..))
        | (Exp1(..), Dere(..))
        | (Exp1(..), Cntr(..)) => true,
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
        (Left((out,)), With((ctx,), l, _)) => {
            net.plug_box(l, vec![out, ctx]);
        }
        (Right((out,)), With((ctx,), _, r)) => {
            net.plug_box(r, vec![out, ctx]);
        }
        (Exp0(_), Weak((oc,), ob)) => {
            net.plug_box(ob, vec![oc]);
        }
        (Exp0(ebox), Dere((out,))) => {
            net.plug_box(ebox, vec![out]);
        }
        (Exp0(ebox), Cntr((a,), (b,))) => {
            net.link(Exp0(ebox.clone()).to_tree(), a);
            net.link(Exp0(ebox).to_tree(), b);
        }
        (Exp1((input,), _), Weak((wctx,), wbox)) => net.link(input, Weak((wctx,), wbox).to_tree()),
        (Exp1((input,), ebox), Dere((out,))) => {
            let (a, b) = net.create_wire();
            net.link(input, Dere((a,)).to_tree());
            net.plug_box(ebox, vec![out, b]);
        }
        (Exp1((input,), ebox), Cntr((a,), (b,))) => {
            let (a0, a1) = net.create_wire();
            let (b0, b1) = net.create_wire();
            net.link(input, Cntr((a0,), (b0,)).to_tree());
            net.link(a, Exp1((a1,), ebox.clone()).to_tree());
            net.link(b, Exp1((b1,), ebox.clone()).to_tree());
        }
        _ => {}
    }
}
