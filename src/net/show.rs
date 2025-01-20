use crate::net::Net;

use crate::net::Tree;

use crate::net::SymbolId;

use std::collections::BTreeMap;

use crate::net::VarId;

use crate::net::PartitionOrBox;

fn pick_name(scope: &mut BTreeMap<VarId, String>, id: VarId) -> String {
    if let Some(n) = scope.get(&id) {
        return n.clone();
    }
    let mut number_c = id + 1;
    loop {
        let mut result = String::new();
        let mut number = number_c;
        while number > 0 {
            let remainder = (number - 1) % 26;
            let character = (b'a' + remainder as u8) as char;
            result.insert(0, character);
            number = (number - 1) / 26;
        }
        if scope.values().all(|x| *x != result) {
            scope.insert(id, result.clone());
            break result;
        }
        number_c += 1;
    }
}

impl Net {
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
                    ).unwrap();
                    for j in i {
                        write!(
                            &mut s,
                            "{}",
                            self.show_aux(show_agent, scope, visited, indent, j)
                        ).unwrap();
                    }
                    write!(&mut s, ")").unwrap();
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
                    "[{}]",
                    ports
                        .iter()
                        .map(|i| { self.show_tree(show_agent, scope, visited, indent, i) })
                        .fold(String::new(), |acc, s| {
                            if acc.is_empty() {
                                s // No space before the first string
                            } else {
                                acc + " " + &s // Add a space between the strings
                            }
                        })
                )
            }
            PartitionOrBox::Box(net) => {
                format!("{{\n{}}}", net.show_net(show_agent, scope, indent + 1))
            }
        }
    }
}
