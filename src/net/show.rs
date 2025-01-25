use crate::net::Net;
use crate::util::join_with;

use crate::net::Tree;

use crate::net::SymbolId;

use std::collections::BTreeMap;

use crate::net::VarId;

use crate::net::PartitionOrBox;
use crate::util::pick_name;

impl Net {
    pub fn print_net_simple(&self) {
        let mut scope = std::collections::BTreeMap::new();
        let show_agent = |x| format!("{:?}", x);
        println!("{}", self.show_net(&show_agent, &mut scope, 0));
    }

    pub fn show_net_simple(&self) -> String {
        let mut scope = std::collections::BTreeMap::new();
        let show_agent = |x| format!("{:?}", x);
        format!("{}", self.show_net(&show_agent, &mut scope, 0))
    }
    pub fn show_net(
        &self,
        show_agent: &dyn Fn(SymbolId) -> String,
        scope: &mut BTreeMap<VarId, String>,
        indent: usize,
    ) -> String {
        let mut visited = vec![];
        use std::fmt::Write;
        let mut s = String::new();
        for a in &self.ports {
            write!(
                &mut s,
                "{}{}\n",
                "    ".repeat(indent),
                self.show_tree(show_agent, scope, &mut visited, indent, &a)
            )
            .unwrap();
        }
        for (a, b) in &self.redexes {
            write!(
                &mut s,
                "{}{} = {}\n",
                "    ".repeat(indent),
                self.show_tree(show_agent, scope, &mut visited, indent, &a),
                self.show_tree(show_agent, scope, &mut visited, indent, &b)
            )
            .unwrap();
        }
        s
    }
    pub fn show_tree(
        &self,
        show_agent: &dyn Fn(SymbolId) -> String,
        scope: &mut BTreeMap<VarId, String>,
        visited: &mut Vec<VarId>,
        indent: usize,
        tree: &Tree,
    ) -> String {
        match tree {
            Tree::Agent(symbol, aux) => {
                use std::fmt::Write;
                let mut s = String::new();
                write!(&mut s, "{}", show_agent(symbol.clone())).unwrap();
                let mut i = aux.iter();
                if let Some(e) = i.next() {
                    write!(
                        &mut s,
                        "{}",
                        self.show_aux(show_agent, scope, visited, indent, e)
                    )
                    .unwrap();
                    for j in i {
                        write!(
                            &mut s,
                            "{}",
                            self.show_aux(show_agent, scope, visited, indent, j)
                        )
                        .unwrap();
                    }
                }
                s
            }
            Tree::Var(id) => {
                if let Some(Some(b)) = self.vars.get(id)
                    && !visited.contains(id)
                {
                    visited.push(*id);
                    self.show_tree(show_agent, scope, visited, indent, b)
                } else {
                    pick_name(scope, *id)
                }
            }
        }
    }
    pub fn show_aux(
        &self,
        show_agent: &dyn Fn(SymbolId) -> String,
        scope: &mut BTreeMap<VarId, String>,
        visited: &mut Vec<VarId>,
        indent: usize,
        aux: &PartitionOrBox,
    ) -> String {
        match aux {
            PartitionOrBox::Partition(ports) => {
                format!(
                    "({})",
                    join_with(
                        ports
                            .iter()
                            .map(|i| { self.show_tree(show_agent, scope, visited, indent, i) }),
                        " ".to_string(),
                    )
                )
            }
            PartitionOrBox::Box(net) => {
                format!("[\n{}]", net.show_net(show_agent, scope, indent + 1))
            }
        }
    }
}
