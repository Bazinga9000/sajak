use crate::frontends::load_default_tree;
use nu_plugin::{Plugin, PluginCommand};

use crate::{
    corpus::trie::CorpusTrie,
    nu::{command_sajak::SajakCommand, command_sajak_trie::SajakTrieCommand},
};

pub struct SajakPlugin {
    pub trie: Option<CorpusTrie>,
}

impl SajakPlugin {
    pub fn new() -> SajakPlugin {
        SajakPlugin {
            trie: load_default_tree(),
        }
    }
}

impl Plugin for SajakPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(SajakCommand), Box::new(SajakTrieCommand)]
    }
}
