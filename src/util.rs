use std::collections::BTreeMap;

use crate::net::VarId;

pub fn number_to_string(mut number: usize) -> String {
    let mut result = String::new();
    while number > 0 {
        let remainder = (number - 1) % 26;
        let character = (b'a' + remainder as u8) as char;
        result.insert(0, character);
        number = (number - 1) / 26;
    }
    result
}
pub fn pick_name(scope: &mut BTreeMap<VarId, String>, id: VarId) -> String {
    if let Some(n) = scope.get(&id) {
        return n.clone();
    }
    let mut number = id + 1;
    loop {
        let result = number_to_string(number);
        if scope.values().all(|x| *x != result) {
            scope.insert(id, result.clone());
            break result;
        }
        number += 1;
    }
}

pub fn join_with(a: impl Iterator<Item = String>, joiner: String) -> String {
    a.fold(String::new(), |acc, s| {
        if acc.is_empty() {
            s
        } else {
            acc + &joiner + &s
        }
    })
}
