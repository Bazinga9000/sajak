use std::path::PathBuf;

use clap::Parser;
use nom::Parser as NomParser;

use crate::expr::Expr;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "sajak", version, about = "Search for words and phrases matching a query.", long_about = None)]
pub struct SajakCli {
    /// The query to parse
    pub query: Vec<String>,

    /// The maximum number of outputs.
    #[arg(short = 'r', long, default_value_t = 50)]
    pub max_results: usize,

    /// The maximum number of nodes to search, in millions.
    #[arg(short = 'n', long, default_value_t = 4.0)]
    pub max_nodes: f32,

    /// Save the compiled FST to the given file.
    #[arg(short = 'l', long, default_value_t = false)]
    pub no_loopbacks: bool,

    /// Save the compiled FST to a file.
    #[arg(short = 's', long)]
    pub save_fst: Option<PathBuf>,

    /// Search over the trie saved in the specified file, as opposed to the default.
    #[arg(short = 'c', long)]
    pub corpus: Option<PathBuf>,

    /// Enable debug logs
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

pub fn parse_expr_cli(query: &str) -> Result<Expr, String> {
    match nom::combinator::all_consuming(crate::expr::parse_expr).parse(query) {
        Err(nom::Err::Error(nom::error::Error { input: e, .. })) => {
            Err(format!("Parse error at {}", e))
        }

        Ok((_, expr)) => Ok(expr),
        _ => unreachable!(),
    }
}
