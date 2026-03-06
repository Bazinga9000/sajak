use super::parsing::{parse_node_at};
use super::simple_trie::SimpleTrie;
use nom::{IResult, Parser};
use std::path::PathBuf;

pub struct CorpusTrie {
    pub num_entries: u64,
    pub num_words: u64,
    pub total_word_frequency: u64,
    pub root_frequency_log: f64,
    pub total_word_freq_log: f64,
    root: CorpusNode,
    pub blob: Vec<u8>,
}

impl CorpusTrie {
    // Returns the node at the given offset
    pub fn node_at(&self, offset: usize) -> CorpusNode {
        parse_node_at(offset, &self.blob).unwrap().1
    }

    // The corpus frequency of the path leading to this node
    // Equal to the node's stored frequency minus those of all its children
    fn in_corpus_frequency(&self, node: &CorpusNode) -> u64 {
        node.own_frequency
    }

    // The log of the relative frequency of this node in the **trie**
    // This trie is such that adding a child with frequency n adds n to the frequencies of its parents,
    // and thus the trie has the heap property. This is used to determine in what order to search the trie.
    pub fn search_score(&self, node: &CorpusNode) -> f64 {
        (node.frequency as f64).log10() - self.root_frequency_log
    }

    // The log of the relative corpus frequency of the string terminating at this node
    // For nodes not in the corpus, this will return -inf
    pub fn corpus_score(&self, node: &CorpusNode) -> f64 {
        (self.in_corpus_frequency(node) as f64).log10() - self.total_word_freq_log
    }

    pub fn root(&self) -> CorpusNode {
        self.root.clone()
    }

    pub fn children_of(&self, node: &CorpusNode) -> Vec<CorpusNode> {
        (&node.child_offsets)
            .into_iter()
            .map(|n| self.node_at(*n))
            .collect::<Vec<_>>()
    }

    fn parse_trie(input: &[u8]) -> IResult<&[u8], CorpusTrie> {
        let mut num = nom::number::complete::le_u64;
        let (rest, num_entries) = num.parse(input)?;
        let (rest, num_words) = num.parse(rest)?;
        let (rest, total_word_frequency) = num.parse(rest)?;
        let (blob_slice, root_offset) = nom::combinator::map(num, |x| x as usize).parse(rest)?;
        let root = parse_node_at(root_offset, &blob_slice)?.1;

        Ok((
            &[],
            CorpusTrie {
                num_entries,
                num_words,
                total_word_frequency,
                root_frequency_log: (root.frequency as f64).log10(),
                total_word_freq_log: (total_word_frequency as f64).log10(),
                root,
                blob: blob_slice.to_vec(),
            },
        ))
    }

    pub fn from_file(file: PathBuf) -> Option<CorpusTrie> {
        let bytes = std::fs::read(file).ok()?;
        CorpusTrie::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Option<CorpusTrie> {
        CorpusTrie::parse_trie(bytes).ok().map(|x| x.1)
    }

    pub fn from_simple_trie(st: SimpleTrie) -> CorpusTrie {
        CorpusTrie::from_bytes(&st.serialize()).unwrap()
    }

    pub fn from_counter(c: counter::Counter<String, u64>) -> CorpusTrie {
        CorpusTrie::from_simple_trie(SimpleTrie::from_counter(c))
    }
}

#[derive(Clone)]
pub struct CorpusNode {
    pub label: char,
    pub frequency: u64,
    pub own_frequency: u64,
    pub is_terminal: bool,
    pub child_offsets: Vec<usize>,
}
