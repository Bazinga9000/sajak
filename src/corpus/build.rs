use counter::Counter;
use regex::Regex;
use rustfst::prelude::{ExpandedFst, SerializableFst};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};
use unidecode::unidecode;

use super::simple_trie::SimpleTrie;
use crate::fst_ops::exact_list;

#[derive(Serialize, Deserialize, Debug)]
struct WikipediaArticle {
    id: String,    // would be u64 but wikiextractor outputs ids as strings
    revid: String, // ^
    url: String,
    title: String,
    text: String,
}

const SPACE_CHARACTERS: [char; 4] = ['-', '_', ':', ';'];

fn clean_string(s: &str) -> String {
    let ss = unidecode(
        // de-unicode the string...
        // ...after removing formula_XXXX text caused by math tags
        &Regex::new("formula_[0-9]+").unwrap().replace(s, ""),
    )
    .to_lowercase() // then convert to lowercase
    .replace("amp;", "and") // handle a few miscellanous HTML that the parser didn't catch
    .replace("lt;", "<")
    .replace("gt;", ">")
    .replace(&SPACE_CHARACTERS, " "); // then convert "space-like" characters to spaces
    Regex::new("[^a-z0-9 ]") // and then remove anything that isn't a legal character
        .unwrap()
        .replace(&ss, "")
        .to_string()
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn handle_article(article: WikipediaArticle, counter: &mut Counter<String, u64>) {
    for line in clean_string(&article.text).split("\n") {
        if line == "" {
            continue;
        }
        if line == "see also" {
            break;
        }
        let wv = line
            // remove extraneous spaces
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let l = wv.len();
        for i in 0..l {
            // ones
            counter[&wv[i]] += 1;
        }
        if l < 2 {
            continue;
        }
        for i in 0..l - 1 {
            // twos
            counter[&format!("{} {}", &wv[i], &wv[i + 1])] += 1;
        }
        if l < 3 {
            continue;
        }
        for i in 0..l - 2 {
            // threes
            counter[&format!("{} {} {}", &wv[i], &wv[i + 1], &wv[i + 2])] += 1;
        }
        if l < 4 {
            continue;
        }
        for i in 0..l - 3 {
            // fours
            counter[&format!("{} {} {} {}", &wv[i], &wv[i + 1], &wv[i + 2], &wv[i + 3])] += 1;
        }
        if l < 5 {
            continue;
        }
        for i in 0..l - 4 {
            // fives
            counter[&format!(
                "{} {} {} {} {}",
                &wv[i],
                &wv[i + 1],
                &wv[i + 2],
                &wv[i + 3],
                &wv[i + 4]
            )] += 1;
        }
    }
}

fn remove_rare_entries(counter: &mut Counter<String, u64>, threshold: u64) {
    *counter = counter
        .most_common()
        .into_iter()
        .filter(|(_, v)| *v >= threshold)
        .collect();
}

pub fn build_trie(full_wiki_directory: PathBuf, working_directory: PathBuf) {
    // ******************************************************************************
    // STEP 1: Make Intermediate Counters (one per directory in wikiextractor output)
    // Each intermediate counter will include only phrases that occur twice or more
    // in that directory (since hapax legomena would bloat filesize a ton)
    //
    // Files written: ./sajak_intermediate_counters/counter_XX.json
    // ******************************************************************************
    let count_time = Instant::now();

    // make the intermediate directory path
    let mut intermediate_path = working_directory.clone();
    intermediate_path.push("sajak_intermediate_counters");
    let _ = std::fs::create_dir(&intermediate_path);

    let mut counter_paths = vec![];
    for wiki_dir in std::fs::read_dir(full_wiki_directory)
        .unwrap()
        .filter_map(|x| x.ok())
    {
        let name = wiki_dir.file_name().into_string().unwrap();
        let count_time = Instant::now();

        // Skip directory if the counter_XX.json already exists
        let mut output_path = intermediate_path.clone();
        output_path.push(format!("counter_{}.json", name));
        counter_paths.push(output_path.clone());
        if output_path.exists() {
            println!(
                "Skipping directory {}, counter_{}.json already exists",
                name, name
            );
            continue;
        }

        // Count the directory
        println!("Counting directory {:#?}", name);
        let mut counter = Counter::new();
        let wiki_path = wiki_dir.path();
        for f in std::fs::read_dir(wiki_path).unwrap() {
            let path = f.unwrap().path();
            if let Ok(lines) = read_lines(path) {
                for line in lines.filter_map(|x| x.ok()) {
                    let wa = serde_json::from_str::<WikipediaArticle>(&line).unwrap();
                    if wa.text.is_empty() {
                        continue; // skip redirects
                    }
                    handle_article(wa, &mut counter);
                }
            }
        }

        // remove hapax legomena from the intermediate counter
        remove_rare_entries(&mut counter, 2);
        std::fs::write(output_path, serde_json::to_string(&counter).unwrap()).unwrap();
        println!(
            "Done counting directory {:#?} (took {:.2?})",
            name,
            count_time.elapsed()
        )
    }
    println!("Written intermediates in {:.2?}", count_time.elapsed());

    // *************************************************************
    // STEP 2: Merge the intermediate counters into one metacounter
    // This will only include entries that occur at least five times
    // across ALL intermediate counters
    //
    // Files written: ./metacounter.json
    // *************************************************************
    println!("Merging counters");
    let mut metacounter_path = working_directory.clone();
    metacounter_path.push("metacounter.json");
    let merge_time = Instant::now();

    if metacounter_path.exists() {
        println!("Skipping merge, metacounter.json already exists")
    } else {
        let mut metacounter: Counter<String, u64> = Counter::new();
        for cpath in counter_paths {
            // load the intermediate counter
            let merge_individual_time = Instant::now();
            let mesocounter: Counter<String, u64> =
                serde_json::from_str(&std::fs::read_to_string(&cpath).unwrap()).unwrap();
            metacounter += mesocounter;
            println!(
                "Merged {:#?} ({:.2?})",
                cpath,
                merge_individual_time.elapsed()
            );
        }
        println!("Filtering counter to entries occurring 5 or more times");
        remove_rare_entries(&mut metacounter, 5);
        std::fs::write(
            &metacounter_path,
            serde_json::to_string(&metacounter).unwrap(),
        )
        .unwrap();
        println!("Written metacounter in {:.2?}", merge_time.elapsed());
    }

    // *******************************************************
    // STEP 3: Build the prefix trie and the FSTS from the
    // metacounter.
    //
    // Files written: trie.sjt
    // *******************************************************
    println!("Building trie");
    let metacounter: Counter<String, u64> =
        serde_json::from_str(&std::fs::read_to_string(&metacounter_path).unwrap()).unwrap();

    let build_time = Instant::now();
    let trie = SimpleTrie::from_counter(metacounter);
    let mut trie_output_path = working_directory.clone();
    trie_output_path.push("trie.sjt");
    trie.to_file(trie_output_path);
    println!(
        "Built trie in {:.2?} ({} entries, {} words)",
        build_time.elapsed(),
        trie.num_entries,
        trie.num_words
    );
}

pub fn fst_from(file: &str, out: &str) {
    if let Ok(lines) = read_lines(file) {
        let mut words = vec![];
        for line in lines {
            let cleaned = clean_string(&line.unwrap());
            words.push(cleaned);
        }

        println!(
            "Building FSTS from file {} with {} words",
            file,
            words.len()
        );
        println!("{:?}", words);
        let fst_time = Instant::now();
        let fst = exact_list(&words);
        fst.write(format!("{}.fst", out)).unwrap();
        println!(
            "Built fst in {:#?} ({} states)",
            fst_time.elapsed(),
            fst.num_states()
        );
    } else {
        panic!("Error in opening source file")
    }
}
