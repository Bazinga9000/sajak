use rustfst::{
    algorithms::compose::compose,
    algorithms::concat::concat,
    prelude::{union::union, utils::acceptor, *},
    Semiring,
};
use std::collections::VecDeque;

// Optimize an FST
// Done generally wherever possible
pub fn optimize_fst(fst: &mut VectorFst<TropicalWeight>) {
    optimize(fst).unwrap();
    tr_sort(fst, OLabelCompare {}); // needed to allow composing (for acceptors, read "intersecting") two FSTS
}

// Add a transition from every state to itself taking ' '
pub fn ignore_spaces(fst: &mut VectorFst<TropicalWeight>) {
    for s in fst.states_iter() {
        fst.add_tr(s, Tr::new(' ' as u32, ' ' as u32, TropicalWeight::one(), s))
            .unwrap();
    }
}

// Constructs an FST matching all and only the words in the given list
pub fn exact_list<I, S>(words: I) -> VectorFst<TropicalWeight>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let fsts = words
        .into_iter()
        .map(|w| {
            let path: Vec<u32> = w.as_ref().chars().map(|x| x as u32).collect();
            acceptor(&path, TropicalWeight::one())
        })
        .collect();

    union_many(fsts)
}

// Produces the FST matching the empty string and nothing else
pub fn matches_empty() -> VectorFst<TropicalWeight> {
    let mut empty = VectorFst::<TropicalWeight>::new();
    let state = empty.add_state();
    empty.set_start(state).unwrap();
    empty.set_final(state, TropicalWeight::one()).unwrap();
    empty
}

// Concatenates a vector of FSTs
pub fn concat_many(fst_vec: Vec<VectorFst<TropicalWeight>>) -> VectorFst<TropicalWeight> {
    let mut out = matches_empty();
    for fst in fst_vec {
        concat(&mut out, &fst).unwrap();
        optimize_fst(&mut out);
    }
    out
}

// Unions a vector of FSTs
pub fn union_many(fst_vec: Vec<VectorFst<TropicalWeight>>) -> VectorFst<TropicalWeight> {
    let mut fsts = fst_vec.into_iter().collect::<VecDeque<_>>();
    while fsts.len() > 1 {
        let mut first = fsts.pop_front().unwrap();
        let second = fsts.pop_front().unwrap();
        union(&mut first, &second).unwrap();
        optimize_fst(&mut first);
        fsts.push_back(first);
    }
    fsts.pop_front().unwrap()
}

// Intersects a vector of FSTs
pub fn intersect_many(fst_vec: Vec<VectorFst<TropicalWeight>>) -> VectorFst<TropicalWeight> {
    let mut fsts = fst_vec.into_iter().collect::<VecDeque<_>>();
    while fsts.len() > 1 {
        let first = fsts.pop_front().unwrap();
        let second = fsts.pop_front().unwrap();
        let mut new = compose(first, second).unwrap();
        optimize_fst(&mut new);
        fsts.push_back(new);
    }
    fsts.pop_front().unwrap()
}

// Concatenates n copies of an FST
pub fn n_copies(fst: &VectorFst<TropicalWeight>, n: u64) -> VectorFst<TropicalWeight> {
    let mut out = matches_empty();
    for _ in 0..n {
        concat(&mut out, fst).unwrap();
        optimize_fst(&mut out);
    }
    out
}

// Generate the FST matching this FST or nothing
pub fn optionalize(fst: &mut VectorFst<TropicalWeight>) {
    union(fst, &matches_empty()).unwrap();
    optimize_fst(fst);
}

// deserialize some bytes into an FST, and optionally ignore spaces
pub fn fetch_and_ignore(fst_bytes: &[u8], ignore: bool) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::load(fst_bytes).unwrap();
    if ignore {
        ignore_spaces(&mut fst);
    }
    fst
}

pub fn step_fst(
    fst: &VectorFst<TropicalWeight>,
    cur_state: StateId,
    label: char,
) -> Option<(StateId, char)> {
    for t in fst.get_trs(cur_state).unwrap().into_iter() {
        let Tr {
            ilabel: tr_in,
            olabel,
            weight: _,
            nextstate: next,
        } = t;
        // println!("transition '{}' to {}", *tr_in as u8 as char, next);
        if *tr_in == label as u32 {
            return Some((*next, (*olabel as u8) as char));
        }
    }
    None
}
