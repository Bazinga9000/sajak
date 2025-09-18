use counter::Counter;
use sajak::corpus::trie::{CorpusNode, CorpusTrie};

fn main() {
    let mut counter = Counter::new();

    for (v, f) in [
        ("dog", 5),
        ("dogs", 3),
        ("doggy", 2),
        ("dog bone", 1),
        ("cat", 6),
    ] {
        counter[&v.to_owned()] = f;
    }

    println!("{:#?}", counter);

    let corpus_trie = CorpusTrie::from_counter(counter);
    display_node("".to_owned(), &corpus_trie, &corpus_trie.root());
}

fn display_node(head: String, t: &CorpusTrie, n: &CorpusNode) {
    println!(
        "{}{}: f={} t={} ef={}",
        head,
        n.label,
        n.frequency,
        n.is_terminal,
        t.corpus_score(n),
    );

    let mut new_head = head.clone();
    new_head.push(n.label);
    for c in t.children_of(&n) {
        display_node(new_head.clone(), t, &c);
    }
}
