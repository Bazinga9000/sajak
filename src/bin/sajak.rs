use clap::Parser;
use rustfst::prelude::{ExpandedFst, SerializableFst};
use sajak::cli::{parse_expr_cli, SajakCli};
use sajak::compile::compile_expr;
use sajak::corpus::trie::CorpusTrie;
use sajak::frontends;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = SajakCli::parse();

    let query = cli.query.join(" ");
    let max_results = cli.max_results;
    let max_nodes = cli.max_nodes * 1_000_000.0;
    let enable_loopbacks = !cli.no_loopbacks;
    let save_fst = cli.save_fst;
    let corpus_path = cli.corpus;
    let verbose = cli.verbose;

    if max_results == 0 {
        panic!("Max number of results must be positive, got 0.")
    }

    if max_nodes <= 0.0 {
        panic!(
            "Max number of nodes to search must be positive, got {}",
            max_nodes
        )
    }

    let load_time = Instant::now();

    let trie = match corpus_path {
        Some(p) => {
            CorpusTrie::from_file(p).expect("The provided corpus was nonexistent or invalid.")
        }
        None => frontends::load_default_tree().expect("The default corpus could not be found. Follow the instructions on the GitHub repository to provide the default corpus."),
    };

    if verbose {
        println!("Loaded trie in {:.2?}", load_time.elapsed());
    }

    let parse_time = Instant::now();
    let expr = match parse_expr_cli(&query) {
        Ok(e) => e,
        Err(s) => panic!("{}", s),
    };

    if verbose {
        println!("Parsed expression in {:.2?}", parse_time.elapsed())
    };

    let compile_time = Instant::now();

    let fst = compile_expr(expr);

    if verbose {
        println!(
            "Compiled fst in {:.2?}, ({} states)",
            compile_time.elapsed(),
            fst.num_states()
        );
    }

    if let Some(p) = save_fst {
        if let Err(_) = fst.write(p) {
            println!("Error saving fst.");
        }
    }

    let search_time = Instant::now();
    let search_results = trie
        .perform_search(fst, enable_loopbacks, max_nodes.floor() as u64, max_results)
        .into_iter()
        .map(|r| format!("{}\t{}", r.result, r.score))
        .collect::<Vec<_>>()
        .join("\n");
    println!("{}", search_results);

    if verbose {
        println!("Search took {:.2?}", search_time.elapsed());
    }

    Ok(())
}
