use rustfst::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub enum QueryResult {
    No,
    Prefix,
    Yes,
}

pub fn check_string(fst: &VectorFst<TropicalWeight>, haystack: &str) -> QueryResult {
    if let Some(mut state) = fst.start() {
        'chars: for c in haystack.chars() {
            // println!("walking {} ({}), in state {}", c, c as u8, state);
            for t in fst.get_trs(state).unwrap().into_iter() {
                let Tr {
                    ilabel: tr_in,
                    olabel: _,
                    weight: _,
                    nextstate: next,
                } = t;
                // println!("transition '{}' to {}", *tr_in as u8 as char, next);
                if *tr_in == c as u32 {
                    state = *next;
                    continue 'chars;
                }
            }
            // We only get here if no transition from the current state accepts the current letter.
            // Therefore matching is impossible.
            return QueryResult::No;
        }

        // We've walked the entire string
        if fst.is_final(state).unwrap() {
            // If we're on a final state, we have a match
            QueryResult::Yes
        } else {
            // If not, we at least know this is a *prefix* of a match
            // so we can keep going down this tree branch
            QueryResult::Prefix
        }
    } else {
        // the fst has no start state, and is thus trivially impossible to match
        QueryResult::No
    }
}
