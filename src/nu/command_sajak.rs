use std::path::PathBuf;

use nu_plugin::{PluginCommand, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Signature, SyntaxShape};
use rustfst::prelude::SerializableFst;

use crate::compile::compile_expr;
use crate::corpus::trie::CorpusTrie;

use super::{
    plugin::SajakPlugin,
    plugin_ops::{load_trie_from_file, parse_expr_nu},
    tablify_results::tablify_results,
};

const DEFAULT_MAX_NODES: f64 = 4.0;
const DEFAULT_MAX_RESULTS: i32 = 50;

pub struct SajakCommand;

impl SimplePluginCommand for SajakCommand {
    type Plugin = SajakPlugin;
    fn name(&self) -> &str {
        "sajak"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(PluginCommand::name(self))
            .category(Category::Generators)
            .required("query", SyntaxShape::String, "The query to search for.")
            .named(
                "max_results",
                SyntaxShape::Int,
                format!(
                    "The maximum length of the output table (default {}).",
                    DEFAULT_MAX_RESULTS
                ),
                Some('r'),
            )
            .named(
                "max_nodes",
                SyntaxShape::Float,
                format!(
                    "The maximum number of nodes to search, in millions (default {}).",
                    DEFAULT_MAX_NODES
                ),
                Some('n'),
            )
            .switch(
                "no-loopbacks",
                "Do not consider concatenations of results as valid results.",
                Some('l'),
            )
            .named(
                "save-fst",
                SyntaxShape::Filepath,
                "Save the compiled FST to the given file.",
                Some('s'),
            )
            .named(
                "corpus",
                SyntaxShape::Filepath,
                "Search over the trie saved in the specified file, as opposed to the default.",
                Some('c'),
            )
    }

    fn description(&self) -> &str {
        "Search for words and phrases matching a query."
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        _: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _: &nu_protocol::Value,
    ) -> Result<nu_protocol::Value, nu_protocol::LabeledError> {
        // grab the flags
        let max_results = call
            .get_flag::<i32>("max_results")?
            .unwrap_or(DEFAULT_MAX_RESULTS);
        if max_results <= 0 {
            return Err(LabeledError::new("Invalid argument").with_label(
                "max_results must be positive",
                call.get_flag_span("max_results").unwrap(),
            ));
        }

        let max_nodes = call
            .get_flag::<f64>("max_nodes")?
            .unwrap_or(DEFAULT_MAX_NODES)
            * 1_000_000.0;
        if max_nodes <= 0.0 {
            return Err(LabeledError::new("Invalid argument").with_label(
                "max_nodes must be positive",
                call.get_flag_span("max_nodes").unwrap(),
            ));
        }

        let allow_loopbacks = !(call.has_flag("no_loopbacks")?);

        let parsed_expr = parse_expr_nu(&call.positional[0])?;
        let fst = compile_expr(parsed_expr);

        if let Some(path) = call.get_flag::<PathBuf>("save-fst")? {
            if let Err(_) = fst.write(path) {
                return Err(LabeledError::new("I/O Error")
                    .with_label("Error saving FST.", call.get_flag_span("save-fst").unwrap()));
            }
        }

        let search = |t: &CorpusTrie| {
            t.perform_search(
                fst,
                allow_loopbacks,
                max_nodes.floor() as u64,
                max_results as usize,
            )
        };

        let results = if let Some(trie_path) = call.get_flag::<PathBuf>("corpus")? {
            let trie = load_trie_from_file(trie_path, call.head)?;
            search(&trie)
        } else {
            let default_trie = (&plugin.trie).as_ref().ok_or(
                LabeledError::new("No or invalid default corpus").with_label(
                    "The default corpus is either nonexistent or invalid. Follow the instructions on the GitHub repository to produce a valid corpus.", call.head
                )
            )?;
            search(default_trie)
        };

        tablify_results(results, &call.head)
    }
}
