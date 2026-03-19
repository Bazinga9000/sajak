# Sajak

Sajak is a tool that can match regular expression like queries on a corpus generated from all of English Wikipedia, intended for use in puzzlehunts.

It's of the same cloth as [Nutrimatic](https://nutrimatic.org) in that it is able to provide results that do not actually appear in the corpus, but are instead concatenations of multiple phrases that *do* appear in Wikipedia. The combined use of Wikipedia as a source, multi-word phrases, and concatenation means Sajak will be able to provide results like notable names, common idioms, and things never before written in the English langauge (like `my golden octopus powered reinforced steel chair`):
```
> sajak [[my golden octopus powered reinforced steel chair]]
╭───┬──────────────────────────────────────────────────┬────────┬────────┬─────────┬───────┬──────────╮
│ # │                      result                      │ score  │ length │ letters │ words │ scrabble │
├───┼──────────────────────────────────────────────────┼────────┼────────┼─────────┼───────┼──────────┤
│ 0 │ my golden octopus powered reinforced steel chair │ -29.84 │     48 │      42 │     7 │       70 │
╰───┴──────────────────────────────────────────────────┴────────┴────────┴─────────┴───────┴──────────╯
```

The output list is also ordered by corpus frequency, so the top of the list will be populated with more "normal" entries, with the results getting progressively more obscure as results continue. Artificially constructed phrases (like the above example) will be penalized in score compared to phrases naturally appearing in the corpus.

Sajak is provided with the following front-ends:
- A **Nushell plugin** to leverage Nushell's built-in table functionality and provide for some additional query power.
- A **Standard CLI** for use in other shells.
- An **HTTP Server** for use as an API.

The Nushell plugin allows sajak to take advantage of Nushell's built-in table functuality and provide for some additional query power. For example:

"Take the 1000 most frequent seven letter strings where letters 1,3,4,6 are consonants and leters 2,5,7 are vowels, and give me the 20 with highest scrabble scores"
```
> sajak CVCCVCV -r 1000 | sort-by scrabble -r | take 20
╭────┬──────────┬───────┬────────┬─────────┬───────┬──────────╮
│  # │  result  │ score │ length │ letters │ words │ scrabble │
├────┼──────────┼───────┼────────┼─────────┼───────┼──────────┤
│  0 │ zip code │ -5.47 │      8 │       7 │     2 │       21 │
│  1 │ hawkeye  │ -5.88 │      7 │       7 │     1 │       20 │
│  2 │ fanzine  │ -5.96 │      7 │       7 │     1 │       19 │
│  3 │ for size │ -5.92 │      8 │       7 │     2 │       19 │
│  4 │ was size │ -5.74 │      8 │       7 │     2 │       19 │
│  5 │ hockey a │ -5.72 │      8 │       7 │     2 │       19 │
│  6 │ mendoza  │ -5.32 │      7 │       7 │     1 │       19 │
│  7 │ buckeye  │ -5.92 │      7 │       7 │     1 │       18 │
│  8 │ may make │ -5.88 │      8 │       7 │     2 │       18 │
│  9 │ benzene  │ -5.87 │      7 │       7 │     1 │       18 │
│ 10 │ him have │ -5.85 │      8 │       7 │     2 │       18 │
│ 11 │ had june │ -5.84 │      8 │       7 │     2 │       18 │
│ 12 │ kicked a │ -5.75 │      8 │       7 │     2 │       18 │
│ 13 │ way home │ -5.72 │      8 │       7 │     2 │       18 │
│ 14 │ to prize │ -5.70 │      8 │       7 │     2 │       18 │
│ 15 │ gonzaga  │ -5.52 │      7 │       7 │     1 │       18 │
│ 16 │ punjabi  │ -5.24 │      7 │       7 │     1 │       18 │
│ 17 │ mixtape  │ -5.24 │      7 │       7 │     1 │       18 │
│ 18 │ may have │ -4.15 │      8 │       7 │     2 │       18 │
│ 19 │ new june │ -5.97 │      8 │       7 │     2 │       17 │
╰────┴──────────┴───────┴────────┴─────────┴───────┴──────────╯
```
## Query Syntax
The syntax of Sajak queries is also like Nutrimatic's (so most puzzlehunting tools/functions will work on Sajak without adjustment), with some extentions not present in Nutrimatic.

All results are normalized such that only lowercase letters, numbers, and spaces are present. This allows uppercase letters to be used for additional functionality.

The full query syntax is as follows:
- `.` matches any legal character (equivalent to the standard regex `[a-z0-9 ]`).
- `_` matches any character except a space (as in `[a-z0-9]`).
- `#` matches a digit (equivalent to `[0123456789]`).
- `-` matches an optional space (` ?`).
- `?`, `+`, `*`, `|`, `()`, `[]`, and `{}` work as in standard regex (though character classes do not allow ranges like `[a-z]`, only finite sets of characters like `[abcde]` or inverted classes like `[^abcde]`)
- `&` matches the intersection of two expressions (`[abcde]&V` is equivalent to `[ae]`)
- `~` after an expression reverses it. (e.g `P~` matches `h`, `eh`, `il`, ...)
- `A`, `C`, and `V` match any letter, a consonant, or a vowel, respectively.
- `P` matches an elemental symbol (`h`, `he`, `li`, ..., `og`).
- `S` matches a U.S. state abbreviation (`al`, `ak`, `ar`, ..., `wy`).
- `W` matches a word that occurs in [this exact word list](https://github.com/dwyl/english-words/blob/master/words_alpha.txt) (`the`, `of`, `and`, ...). \
  *This is several hundred thousand words, and may take anywhere from a bit longer to* considerably *longer to produce results, depending on how complex your expression is elsewhere.*
- Surrounding a query in `<angle brackets>` matches all anagrams of its components. \
 These can be individual letters (`<longed>` matches `golden`), or can be more complicated expressions (`<(a|b)(c|d)CCV>` matches words like `based`, `place`, and `named`). \
  *Patterns with extremely complicated anagrams (>10 expressions) may be slow to produce results. See "How it works" for more info.*
- In all queries, **spaces in potential matches are ignored** unless you surround all or part of the query with ``[[double brackets]]``.
**This is a difference from Nutrimatic**, which uses `"quoted strings"` for this purpose. \
 *(This change is to allow quotes to be used to wrap the entire query in the CLI when it contains otherwise shell-relevant characters like `|`, and so that we can avoid having to escape quotes in the query)*

Note that the by-default ignoring of spaces makes tokens like `-` and `_` redundant outside of double-bracketed expressions, but they are allowed anywhere.

## NixOS Installation
The provided Nix flake will install and configure the various sajak binaries for you. It will also fetch the latest default corpus from this repository and use it as its default automatically.

You can try out the basic CLI without installing them with the following command:
```
nix run github:Bazinga9000/sajak
```

Alternatively, to include Sajak in an NixOS configuration, add this flake to your inputs:
```nix
inputs.sajak = {
    url = "github:Bazinga9000/sajak";
    inputs.nixpkgs.follows = "nixpkgs";
}; 
```
And either add the desired packages from inputs:
```nix
environment.systemPackages = [
    inputs.baz9k-pkgs.packages.${pkgs.stdenv.hostPlatform.system}.sajak
]
```
Or enable the provided overlay:
```nix
nixpkgs.overlays = [ inputs.baz9k-pkgs.overlays.default ];
environment.systemPackages = [
    sajak
];
```

The Nix package for Sajak is automatically built and cached daily with [Garnix](https://garnix.io/). To avoid local builds, add the following to your nix configuration (or append to your existing `substituters` and `trusted-public-keys` if they already exist):
```nix
nix.settings = {
  substituters = [ "https://cache.garnix.io" ];
  trusted-public-keys = [ "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" ];
};
```


## Installation and Usage Guide
### Nushell Plugin
Build the `nu_plugin_sajak` binary using Cargo and then run the Nu command
```
plugin add nu_plugin_sajak
```
The Nushell plugin exposes `sajak` and `sajak-mkfst`. Run them with `-h` for usage information.

### Standard CLI
Build the `sajak` binary using Cargo. This binary works as a standalone interface for querying. If you need to make FSTs, use the Nushell inferface. Use `sajak -h` for usage information.

### HTTP Server
Build the `sajak_http` binary using Cargo and run it. This will expose an HTTP server on the port specified in the `PORT` environment variable (1983 by default). The server suppports the following endpoints (mostly matching the options of the Nushell plugin):
- GET `/health`, returns a simple health and version check.
- GET `/query`, performs the sajak query using the following JSON parameters: 
  - `query`, a required string in the above query syntax.
  - `max_nodes`, a float specifying the number of nodes to search in millions. Defaults to `4`.
  - `max_results`, a positive `u16` specifying the maximum number of results to display. Defaults to `10`.
  - `enable_loopbacks`, a boolean describing whether to allow concatenations to be considered valid. Defaults to `true`.

A valid request will return a JSON array of dicts with the following keys:
- `result`, the result of the query
- `score`, the frequency score
- `length`, the total length of the result
- `length_nospace`, the total length of the result, excluding spaces 
- `num_words`, the number of words in the result
- `scrabble`, the Scrabble score of the result


## Getting required files

Sajak requires a trie file to be built containing the corpus data. It is not packaged with the source or binary, since the file is >1GB in size.

Use either of the two below methods to produce the `trie.sjt` file and then place this file in your user's **data** directory, in `sajak`.
- On Linux, this is `/home/USERNAME/.local/share/sajak` (unless you have changed `$XDG_DATA_HOME`, in which case you know what you're doing)
- On MacOS, this is `/Users/USERNAME/Library/Application Support/sajak`
- On Windows, this is `C:\Users\USERNAME\AppData\Roaming`

Alternatively, you can specify a different default directory using the `SAJAK_DEFAULT_TRIE` environment variable. All frontends use this for their default corpus if set.

If you've installed the Nushell plugin before doing this, you will need to wait for Nushell to
[garbage collect](https://www.nushell.sh/book/plugins.html#plugin-garbage-collector) it, which will happen 10 seconds after use by default,
for the addition of the file to take effect (as Nushell caches plugins).

### Option 1: Acquiring a prebuilt trie (recommended)

You can find premade versions of these files used by me in the Releases tab.

These files were generated based on the Wikipedia dump on **October 20, 2024**.

### Option 2: Generate your own trie
*(This will require >30GB of RAM and ~50GB of disk space! It will also require the nushell plugin frontend, as that is the only frontend that implements exposes the corpus-building functionality.)*

If you wish to build your own prefabs instead of using the provided one (if, say, the provided set is too out of date for your liking), perform the following steps:

1. Download a Wikipedia dump. It should be of the form `enwiki-20241020-pages-articles.xml.bz2`. Do **not** use the multistream version.
2. Download [attardi's wikiextractor](<https://github.com/attardi/wikiextractor>) and extract the dump into a directory of your choosing:
```
wikiextractor -o <output-directory-of-your-choosing> --no-templates --json <your-pages-articles-xml>
```
If you're on a newer version of python and get errors, you may need to specifically install this commit (this is what I did):
```
pip install git+https://github.com/attardi/wikiextractor.git@ab8988ebfa9e4557411f3d4c0f4ccda139e18875
```
3. Run
```
sajak-mktrie <directory-you-dumped-to>
```
This will produce several files and directories in the working directory:
- `sajak_intermediate_counters` is a directory containing intermediate data. You can safely delete this unless you expect to rebuild the trie again from the same dump (the program skips generating these files if they exist).
- `metacounter.json` is a JSON map containing the raw frequency data (~1G in size), before it is turned into a trie. This might be worth keeping around if you think you might need a big Wikipedia corpus for other projects.
- `trie.sjt` is the trie itself (~1GB). Put this in your config directory.

## How it Works

Sajak borrows a lot of its technique from Nutrimatic.

Corpus data is stored in a prefix trie.

Expressions are parsed using `nom` and converted into a finite-state transducer using `rustfst`. The FST is optimized after every intermediate compilation step, resulting in generally fast compilation for most "normal" regular expressions. However, the intersection operator (and anagram, which is implemented in terms of it) has the potential to require exponentially many states and thus cause compilation time to blow up with extremely complicated intersections. The `W`, `P`, and `S` atoms have precompiled FSTs baked into the binary that are pulled in wherever they're needed.

The trie is then walked using best-first search by, pruning branches that can't possibly be prefixes of any match. Use of FSTs allows for fast computation of matches (especially when you can store the FST state as the tree is walked), allowing for rapid search after compilation.

The output "score" in the table is the log (base 10) of the frequency of the result divided by the total frequencies of all *individual words* in the corpus. If the result is a concatenation, the score is the sum of the scores (and hence the product of the relative frequences) of the components. (If you're curious, the most common entry in the corpus, `the`, has a score of about `-1.13`, so about 7.4% of Wikipedia is `the`.)

## Future Plans and Contributing

`rustfst` supports finite state *transducers*, which transform their input in addition to matching it, alongside finite state machines. This means that Sajak could in theory have its syntax expanded to query data after being transduced, which would allow for even greater query power. `rustfst`'s `LazyFst` may also help with compile time for the more expensive expressions.

Issues, pull requests, optimizations, and general feedback are incredibly welcome. Happy puzzling!
