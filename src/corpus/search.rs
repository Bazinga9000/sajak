use super::trie::{CorpusNode, CorpusTrie};
use crate::fst_ops::step_fst;
use rustfst::{
    prelude::{CoreFst, TropicalWeight, VectorFst},
    StateId,
};
use std::{
    collections::{BinaryHeap, HashSet},
    sync::Arc,
};

impl CorpusTrie {
    pub fn perform_search(
        &self,
        fst: VectorFst<TropicalWeight>,
        allow_loopbacks: bool,
        node_search_limit: u64,
        max_visible_results: usize,
    ) -> Vec<SearchResult> {
        if let Some(start_state) = fst.start() {
            let root = self.root();

            let mut q = BinaryHeap::new();
            q.push(QueueItem {
                result: "".to_owned(),
                prior_corpus_score: 0.0,
                current_search_score: 0.0,
                fst: Arc::new(fst),
                fst_state: start_state,
                trie_node: root,
            });

            let mut res_q = BinaryHeap::new();
            let mut seen = HashSet::new();
            let mut nodes_searched = 0;
            while let Some(qi) = q.pop() {
                if nodes_searched >= node_search_limit {
                    dbg!(nodes_searched);
                    dbg!(q.len());
                    break;
                }

                nodes_searched += 1;
                // println!("{}: tss={} pcs={} css={} ccs={}", qi.result, qi.total_search_score(), qi.prior_corpus_score, qi.current_search_score, self.corpus_score(&qi.trie_node));

                for child in qi.step(self, allow_loopbacks) {
                    q.push(child);
                }

                if qi.is_accepted() // FST is on a final state
                && qi.is_in_corpus() // state is terminal
                // && !qi.result.ends_with(" ") // does not end with a space (no root)
                && !seen.contains(&qi.result)
                {
                    // not previously seen
                    seen.insert(qi.result.clone());
                    res_q.push(SearchResult::from_queueitem(qi, self))
                }
            }
            let mut results = vec![];
            while let Some(res) = res_q.pop() {
                if results.len() >= max_visible_results {
                    break;
                }
                results.push(res);
            }

            results
        } else {
            vec![]
        }
    }
}

struct QueueItem {
    result: String,
    prior_corpus_score: f64,
    current_search_score: f64,

    fst: Arc<VectorFst<TropicalWeight>>,
    fst_state: StateId,
    trie_node: CorpusNode,
}

impl QueueItem {
    fn is_in_corpus(&self) -> bool {
        self.trie_node.is_terminal
    }

    fn is_accepted(&self) -> bool {
        self.fst.is_final(self.fst_state).unwrap()
    }

    fn step(&self, trie: &CorpusTrie, allow_loopbacks: bool) -> Vec<QueueItem> {
        let mut new_items = vec![];

        for child in trie.children_of(&self.trie_node) {
            if let Some((next_state, _)) = step_fst(self.fst.as_ref(), self.fst_state, child.label)
            {
                let mut new_result = self.result.clone();
                new_result.push(child.label);
                new_items.push(QueueItem {
                    prior_corpus_score: self.prior_corpus_score,
                    current_search_score: trie.search_score(&child),
                    result: new_result,

                    fst: self.fst.clone(),
                    trie_node: child,
                    fst_state: next_state,
                });
            }
        }

        if allow_loopbacks && self.is_in_corpus() {
            if let Some((next_state, _)) = step_fst(&self.fst, self.fst_state, ' ') {
                let mut new_result = self.result.clone();
                new_result.push(' ');
                let root = trie.root();
                new_items.push(QueueItem {
                    prior_corpus_score: self.prior_corpus_score
                        + trie.corpus_score(&self.trie_node),
                    current_search_score: 0.0, // root, by definition, has search score 0 (the maximum)
                    result: new_result,

                    fst: self.fst.clone(),
                    fst_state: next_state,
                    trie_node: root,
                })
            }
        }
        new_items
    }

    pub fn total_search_score(&self) -> f64 {
        self.prior_corpus_score + self.current_search_score
    }
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.result == other.result && self.total_search_score() == other.total_search_score()
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self
            .total_search_score()
            .partial_cmp(&other.total_search_score())
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.result.partial_cmp(&other.result)
    }
}

impl Eq for QueueItem {}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub result: String,
    pub score: f64,
    pub length: u32,
    pub length_nospace: u32,
    pub num_words: u32,
    pub scrabble: u32,
}

impl SearchResult {
    fn from_queueitem(qi: QueueItem, t: &CorpusTrie) -> SearchResult {
        let current_corpus_score = t.corpus_score(&qi.trie_node);
        SearchResult {
            score: qi.prior_corpus_score + current_corpus_score,
            length: qi.result.len() as u32,
            length_nospace: SearchResult::length_nospace(&qi.result),
            num_words: SearchResult::num_words(&qi.result),
            scrabble: SearchResult::scrabble_score(&qi.result),

            result: qi.result,
        }
    }

    fn scrabble_score(s: &str) -> u32 {
        s.chars()
            .map(|c| match c {
                'e' | 'a' | 'i' | 'o' | 'n' | 'r' | 't' | 'l' | 's' | 'u' => 1,
                'd' | 'g' => 2,
                'b' | 'c' | 'm' | 'p' => 3,
                'f' | 'h' | 'v' | 'w' | 'y' => 4,
                'k' => 5, // isn't it weird how k is just by itself in the 5 point zone?
                'j' | 'x' => 8,
                'q' | 'z' => 10,
                _ => 0,
            })
            .sum()
    }

    fn length_nospace(s: &str) -> u32 {
        s.chars()
            .map(|c| match c {
                ' ' => 0,
                _ => 1,
            })
            .sum()
    }

    fn num_words(s: &str) -> u32 {
        s.split_whitespace().count() as u32
    }
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.result == other.result && self.score == other.score
    }
}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.score.partial_cmp(&other.score) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.result.partial_cmp(&other.result)
    }
}

impl Eq for SearchResult {}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
