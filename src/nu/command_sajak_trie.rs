use crate::corpus::build::build_trie;
use nu_plugin::{PluginCommand, SimplePluginCommand};
use nu_protocol::{Category, Signature, SyntaxShape, Value};

use super::plugin::SajakPlugin;

pub struct SajakTrieCommand;

impl SimplePluginCommand for SajakTrieCommand {
    type Plugin = SajakPlugin;
    fn name(&self) -> &str {
        "sajak-mktrie"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(PluginCommand::name(self))
            .category(Category::Generators)
            .required(
                "source",
                SyntaxShape::Filepath,
                "The directory containing the output Wikiextractor data.",
            )
    }

    fn description(&self) -> &str {
        "Generate a Sajak corpus trie from an extracted Wikipedia dump. See the github's README for more information."
    }

    fn run(
        &self,
        _: &Self::Plugin,
        engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _: &nu_protocol::Value,
    ) -> Result<nu_protocol::Value, nu_protocol::LabeledError> {
        let path = call.positional[0].to_path()?;
        let current_dir = std::path::PathBuf::from(engine.get_current_dir()?);
        build_trie(path, current_dir);

        Ok(Value::Nothing {
            internal_span: call.head,
        })
    }
}
