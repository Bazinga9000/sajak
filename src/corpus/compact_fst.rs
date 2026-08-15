use rustfst::{
    fst_traits::{ExpandedFst, StateIterator},
    prelude::CoreFst,
    prelude::{TropicalWeight, VectorFst},
    StateId, Tr,
};

// Precomputed FST transitions for O(1) lookup.
// The legal alphabet is the 38 characters from CHAR_IDS in parsing.rs:
// '\0', '0'-'9', 'a'-'z', ' '.
const ALPHABET_SIZE: usize = 38;

fn char_to_pos(c: char) -> Option<usize> {
    match c {
        '\0' => Some(0),
        '0'..='9' => Some(1 + (c as u8 - b'0') as usize),
        'a'..='z' => Some(11 + (c as u8 - b'a') as usize),
        ' ' => Some(37),
        _ => None,
    }
}

fn label_to_pos(label: u32) -> Option<usize> {
    let c = char::from_u32(label)?;
    char_to_pos(c)
}

#[derive(Clone)]
pub struct CompactFst {
    tables: Vec<[Option<(StateId, char)>; ALPHABET_SIZE]>,
    pub start: StateId,
    finals: Vec<bool>,
}

impl CompactFst {
    pub fn from_fst(fst: &VectorFst<TropicalWeight>) -> Self {
        let num_states = fst.num_states() as usize;
        let mut tables = vec![[None; ALPHABET_SIZE]; num_states];
        let mut finals = vec![false; num_states];

        for s in fst.states_iter() {
            finals[s as usize] = fst.is_final(s).unwrap();
            for tr in fst.get_trs(s).unwrap().into_iter() {
                let Tr {
                    ilabel,
                    olabel,
                    nextstate,
                    ..
                } = tr;
                if let Some(pos) = label_to_pos(*ilabel) {
                    tables[s as usize][pos] = Some((*nextstate, (*olabel as u8) as char));
                }
            }
        }

        CompactFst {
            tables,
            start: fst.start().unwrap(),
            finals,
        }
    }

    pub fn step(&self, cur_state: StateId, label: char) -> Option<(StateId, char)> {
        match char_to_pos(label) {
            Some(pos) => self.tables[cur_state as usize][pos],
            None => None,
        }
    }

    pub fn is_final(&self, state: StateId) -> bool {
        self.finals[state as usize]
    }
}
