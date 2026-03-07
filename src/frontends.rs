use crate::corpus::trie::CorpusTrie;
use dirs::data_dir;
use std::path::PathBuf;

// Fetch the default trie path. Uses the env var SAJAK_DEFAULT_TRIE if set, otherwise defaults to
// Win - %AppData%/sajak/tire.sjt
// Mac - $HOME/Library/Application Support/sajak/trie.sjt
// Linux - $HOME/.local/share/sajak/trie.sjt
pub fn default_trie_path() -> PathBuf {
    let key = "SAJAK_DEFAULT_TRIE";
    match std::env::var_os(key) {
        Some(path) => PathBuf::from(path),
        None => {
            let mut default_trie_path = data_dir().unwrap();
            default_trie_path.push("sajak");
            default_trie_path.push("trie.sjt");
            default_trie_path
        }
    }
}

pub fn load_default_tree() -> Option<CorpusTrie> {
    CorpusTrie::from_file(default_trie_path())
}
