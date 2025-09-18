#[derive(Debug)]
pub struct SimpleNode {
    pub label: char,
    pub frequency: u64,
    pub is_terminal: bool,
    pub children: Vec<SimpleNode>,
}

impl SimpleNode {
    pub fn new(label: char) -> SimpleNode {
        SimpleNode {
            label,
            frequency: 0,
            is_terminal: false,
            children: vec![],
        }
    }

    fn insert(&mut self, s: &str, frequency: u64) {
        self.frequency += frequency;
        match s.chars().nth(0) {
            None => {
                self.is_terminal = true;
            }
            Some(next) => {
                let remainder = &s[1..];
                for child in self.children.iter_mut() {
                    if child.label == next {
                        child.insert(remainder, frequency);
                        return;
                    }
                }

                // no children with given label, make a new one
                let mut new_child = SimpleNode::new(next);
                new_child.insert(remainder, frequency);
                self.children.push(new_child);
            }
        }
    }
}

#[derive(Debug)]
pub struct SimpleTrie {
    pub num_entries: u64,
    pub num_words: u64,
    pub total_word_frequency: u64,
    pub root: SimpleNode,
}

impl SimpleTrie {
    pub fn new() -> SimpleTrie {
        SimpleTrie {
            num_entries: 0,
            num_words: 0,
            total_word_frequency: 0,
            root: SimpleNode::new('\0'),
        }
    }

    pub fn insert(&mut self, entry: &str, frequency: u64) {
        // end temp
        self.num_entries += 1;
        if !entry.contains(' ') {
            self.num_words += 1;
            self.total_word_frequency += frequency;
        }
        self.root.insert(entry, frequency);
    }

    pub fn from_counter(c: counter::Counter<String, u64>) -> SimpleTrie {
        let mut st = SimpleTrie::new();
        for (k, v) in c.iter() {
            st.insert(k, *v);
        }
        st
    }
}
