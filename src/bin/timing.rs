use dirs::data_dir;
use rustfst::prelude::ExpandedFst;
use sajak::compile::compile_expr;
use sajak::expr::parse_expr;
use std::time::Instant;
const TEST_EXPR: &str = "<het><ral><seg><tan><rut><bla><oody><afl><ndi><cin><awe><ter>";

fn main() {
    let load_time = Instant::now();

    let mut default_trie_path = data_dir().unwrap();
    default_trie_path.push("sajak");
    default_trie_path.push("trie.sjt");

    let wt = sajak::corpus::trie::CorpusTrie::from_file(default_trie_path).unwrap();
    println!("Loaded trie in {:.2?}", load_time.elapsed());

    let parse_time = Instant::now();
    let expr = parse_expr(TEST_EXPR).unwrap().1;
    println!("Parsed expression in {:.2?}", parse_time.elapsed());

    let compile_time = Instant::now();
    let fst = compile_expr(expr);
    println!(
        "Compiled fst in {:.2?}, ({} states)",
        compile_time.elapsed(),
        fst.num_states()
    );

    let search_time = Instant::now();
    let search_results = wt
        .perform_search(fst, true, 4_000_000, 100)
        .into_iter()
        .map(|r| format!("{}\t{}", r.result, r.score))
        .collect::<Vec<_>>()
        .join("\n");
    println!("{}", search_results);
    println!("Search took {:.2?}", search_time.elapsed());
}
