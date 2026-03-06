use super::simple_trie::{SimpleNode, SimpleTrie};
use std::path::PathBuf;

fn write_efficient_integer(out: &mut Vec<u8>, n: u64) {
    if n < 249 {
        // If n is small, write it as is
        out.push(n as u8);
    } else {
        for (pow, num_bytes) in [(16, 2), (24, 3), (32, 4), (40, 5), (48, 6), (56, 7)] {
            if n < 2u64.pow(pow) {
                out.push(247 + num_bytes);
                out.extend(&n.to_le_bytes()[..num_bytes as usize]);
                return;
            }
        }

        // If we got here, we need all 8 bytes
        // What kind of corpus are you making that has quintillions of entries?
        out.push(255);
        out.extend(&n.to_le_bytes());
    }
}

impl SimpleNode {
    fn get_label_byte(&self) -> u8 {
        let has_children = if !self.children.is_empty() { 1 } else { 0 };
        let is_terminal = if self.is_terminal { 1 } else { 0 };
        let label = match self.label {
            // I'm sorry for this
            // At least it's explicit
            '\0' => 0,
            '0' => 1,
            '1' => 2,
            '2' => 3,
            '3' => 4,
            '4' => 5,
            '5' => 6,
            '6' => 7,
            '7' => 8,
            '8' => 9,
            '9' => 10,
            'a' => 11,
            'b' => 12,
            'c' => 13,
            'd' => 14,
            'e' => 15,
            'f' => 16,
            'g' => 17,
            'h' => 18,
            'i' => 19,
            'j' => 20,
            'k' => 21,
            'l' => 22,
            'm' => 23,
            'n' => 24,
            'o' => 25,
            'p' => 26,
            'q' => 27,
            'r' => 28,
            's' => 29,
            't' => 30,
            'u' => 31,
            'v' => 32,
            'w' => 33,
            'x' => 34,
            'y' => 35,
            'z' => 36,
            ' ' => 37,
            _ => 0, // In case something weird gets through the corpus building
        };

        (has_children << 0) + (is_terminal << 1) + (label << 2)
    }
    fn write(&self, out: &mut Vec<u8>) -> u64 {
        // write all children first, collecting their offsets
        let mut sort_indices = (0..self.children.len()).collect::<Vec<_>>();
        sort_indices.sort_by(|i, j| self.children[*j].frequency.cmp(&self.children[*i].frequency));
        // sort by descending frequency
        let child_offsets: Vec<_> = (sort_indices).iter().map(|c| self.children[*c].write(out)).collect();

        // get the current offset
        let node_offset = out.len() as u64;

        // write label and frequency
        out.push(self.get_label_byte());
        write_efficient_integer(out, self.frequency);

        let own_frequency = self.frequency - self.children.iter().map(|c| c.frequency).sum::<u64>();
        write_efficient_integer(out, own_frequency);

        if !self.children.is_empty() {
            // write the number of children
            out.push(self.children.len() as u8);

            // write the relative offset of all children
            for o in child_offsets
                .iter()
                .map(|child_offset| node_offset - child_offset)
            {
                write_efficient_integer(out, o);
            }
        }

        node_offset
    }
}

impl SimpleTrie {
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = vec![];
        // Write the known quantities
        out.extend(self.num_entries.to_le_bytes());
        out.extend(self.num_words.to_le_bytes());
        out.extend(self.total_word_frequency.to_le_bytes());
        // Push a placeholder for the root offset
        let root_offset_place = out.len();
        out.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
        // Write all the nodes and get the actual root offset (from the start of the blob)
        let blob_start_point = out.len();
        let true_root_offset = self.root.write(&mut out) - blob_start_point as u64;
        // Go back and fill in the root offset
        for (n, byte) in true_root_offset.to_le_bytes().iter().enumerate() {
            out[root_offset_place + n] = *byte;
        }

        out
    }

    pub fn to_file(&self, file: PathBuf) {
        std::fs::write(file, self.serialize()).unwrap();
    }
}
