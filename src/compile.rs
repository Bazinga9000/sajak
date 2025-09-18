use super::expr::Expr;
use super::fst_ops::*;
use closure::closure;
use rustfst::{
    prelude::{
        algorithms::reverse, compose::compose, concat::concat, union::union, utils::acceptor, *,
    },
    Semiring,
};

// const ELEMENTS : [&str; 118] = ["h", "he", "li", "be", "b", "c", "n", "o", "f", "ne", "na", "mg", "al", "si", "p", "s", "cl", "ar", "k", "ca", "sc", "ti", "v", "cr", "mn", "fe", "co", "ni", "cu", "zn", "ga", "ge", "as", "se", "br", "kr", "rb", "sr", "y", "zr", "nb", "mo", "tc", "ru", "rh", "pd", "ag", "cd", "in", "sn", "sb", "te", "i", "xe", "cs", "ba", "la", "ce", "pr", "nd", "pm", "sm", "eu", "gd", "tb", "dy", "ho", "er", "tm", "yb", "lu", "hf", "ta", "w", "re", "os", "ir", "pt", "au", "hg", "tl", "pb", "bi", "po", "at", "rn", "fr", "ra", "ac", "th", "pa", "u", "np", "pu", "am", "cm", "bk", "cf", "es", "fm", "md", "no", "lr", "rf", "db", "sg", "bh", "hs", "mt", "ds", "rg", "cn", "nh", "fl", "mc", "lv", "ts", "og"];
// const US_STATES : [&str; 50] = ["al", "ak", "az", "ar", "ca", "co", "ct", "de", "fl", "ga", "hi", "id", "il", "in", "ia", "ks", "ky", "la", "me", "md", "ma", "mi", "mn", "ms", "mo", "mt", "ne", "nv", "nh", "nj", "nm", "ny", "nc", "nd", "oh", "ok", "or", "pa", "ri", "sc", "sd", "tn", "tx", "ut", "uv", "va", "wa", "wv", "wi", "wy"];

fn compile_quote_flag(expr: Expr, quoted: bool) -> VectorFst<TropicalWeight> {
    let comp = |x| compile_quote_flag(x, quoted);
    let fetch = |x| fetch_and_ignore(x, !quoted);
    match expr {
        Expr::Literal(cs) => {
            let path: Vec<u32> = cs.iter().map(|x| *x as u32).collect();
            let mut fst = acceptor(&path, TropicalWeight::one());
            if !quoted {
                ignore_spaces(&mut fst);
            }
            optimize_fst(&mut fst);
            return fst;
        }
        Expr::Any => {
            return comp(Expr::OneOf(
                "abcdefghijklmnopqrstuvwxyz0123456789 ".chars().collect(),
            ))
        }
        Expr::Nonspace => {
            return comp(Expr::OneOf(
                "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect(),
            ))
        }
        Expr::Alpha => return comp(Expr::OneOf("abcdefghijklmnopqrstuvwxyz".chars().collect())),
        Expr::Consonant => return comp(Expr::OneOf("bcdfghjklmnpqrstvwxyz".chars().collect())),
        Expr::Vowel => return comp(Expr::OneOf("aeiou".chars().collect())),
        Expr::Digit => return comp(Expr::OneOf("0123456789".chars().collect())),
        Expr::OptionalSpace => return comp(Expr::Question(Box::new(Expr::Literal(vec![' '])))),
        Expr::Word => return fetch(include_bytes!("../prefab_fsts/words.fst")),
        Expr::Element => return fetch(include_bytes!("../prefab_fsts/elements.fst")),
        Expr::USState => return fetch(include_bytes!("../prefab_fsts/us_states.fst")),
        Expr::OneOf(chars) => {
            let one = TropicalWeight::one();
            let mut fst = VectorFst::<TropicalWeight>::new();
            let start = fst.add_state();
            let end = fst.add_state();
            fst.set_start(start).unwrap();
            fst.set_final(end, one).unwrap();
            for c in chars {
                fst.add_tr(start, Tr::new(c as u32, c as u32, one, end))
                    .unwrap();
            }
            if !quoted {
                ignore_spaces(&mut fst);
            }
            optimize_fst(&mut fst);
            return fst;
        }
        Expr::NoneOf(chars) => {
            return comp(Expr::OneOf(
                "abcdefghijklmnopqrstuvwxyz0123456789 "
                    .chars()
                    .filter(|c| !chars.contains(c))
                    .collect(),
            ))
        }
        Expr::Or(a, b) => {
            let mut out = comp(*a);
            let rhs = comp(*b);
            union(&mut out, &rhs).unwrap();
            optimize_fst(&mut out);
            return out;
        }
        Expr::And(a, b) => {
            let lhs = comp(*a);
            let rhs = comp(*b);
            // the rustfst "compose" function acts like intersection on acceptors (which all FSTs produced by sajak are (for now))
            let mut out = compose(lhs, rhs).unwrap();
            optimize_fst(&mut out);
            return out;
        }
        Expr::Star(e) => {
            let mut fst = comp(*e);
            closure(&mut fst, closure::ClosureType::ClosureStar);
            optimize_fst(&mut fst);
            return fst;
        }
        Expr::Question(e) => {
            let mut fst = comp(*e);
            optionalize(&mut fst);
            return fst;
        }
        Expr::Plus(e) => {
            let mut fst = comp(*e);
            closure(&mut fst, closure::ClosureType::ClosurePlus);
            optimize_fst(&mut fst);
            return fst;
        }
        Expr::Reverse(e) => {
            let fst = comp(*e);
            let mut out = reverse(&fst).unwrap();
            optimize_fst(&mut out);
            return out;
        }
        Expr::NCopies(e, n) => {
            let compiled = comp(*e);
            return n_copies(&compiled, n);
        }
        Expr::Range(e, mn, mx) => {
            let mut compiled = comp(*e);
            let mut out = n_copies(&compiled, mn);
            match mx {
                None => closure(&mut compiled, closure::ClosureType::ClosureStar), // compiled is now e*
                Some(n) => {
                    if mn > n {
                        panic!("Range query minimum greater than its maximum");
                    } else {
                        optionalize(&mut compiled);
                        compiled = n_copies(&compiled, n - mn); // compiled is now (e?){max - min}
                    }
                }
            }
            concat(&mut out, &compiled).unwrap();
            optimize_fst(&mut out);
            return out;
        }
        Expr::Quote(e) => return compile_quote_flag(*e, true),
        Expr::Anagram(es) => return compile_anagram(es, quoted),
        Expr::Sequence(es) => {
            let mut fsts = es.into_iter().map(comp);
            let mut out = fsts.next().unwrap();
            for fst in fsts {
                concat(&mut out, &fst).unwrap();
                optimize_fst(&mut out);
            }
            return out;
        }
    }
}

// A port of Nutrimatic's anagram algorithm
// TODO: this can probably be heavily optimized
// Example:
// If the query is <aaabbc>
// It will produce an FST matching the intersection of the following:
// (1) (a|b|c)(a|b|c)(a|b|c)(a|b|c)(a|b|c)(a|b|c)
// (2) (b|c)*a(b|c)*a(b|c)*a(b|c)*
//     (a|c)*b(a|c)*b(a|c)*
//     (a|b)*c(a|b)*
// The number of states in the output is exponential (2^N) in the number of inputs. This is because of the internal intersection.
struct AnagramPart {
    fst: VectorFst<TropicalWeight>,
    count: u64,
}

fn compile_anagram(es: Vec<Expr>, quoted: bool) -> VectorFst<TropicalWeight> {
    if es.len() == 0 {
        matches_empty() // empty anagram matches only nothing
    } else if es.len() == 1 {
        compile_quote_flag(es.into_iter().nth(0).unwrap(), quoted) // length 1 anagram, just compile it directly
    } else {
        // compile all the inner expressions, tallying up equivalent ones
        let mut parts: Vec<AnagramPart> = vec![];
        'expr_loop: for e in es {
            let compiled = compile_quote_flag(e, quoted);
            for part in &mut parts {
                // all FSTs are minimized and optimized so equality is equivalence
                if compiled == part.fst {
                    part.count += 1;
                    continue 'expr_loop;
                }
            }

            parts.push(AnagramPart {
                fst: compiled,
                count: 1,
            })
        }

        let mut to_intersect = vec![];
        let num_parts = (&parts).into_iter().map(|p| p.count).sum::<u64>() as u64;

        // (1) match [any part] x [number of elements]
        let any_part = union_many((&parts).into_iter().map(|p| p.fst.clone()).collect());
        let many_any_part = n_copies(&any_part, num_parts);
        to_intersect.push(many_any_part);

        // (2) matches [other parts]* [part i] [other parts]* [part i] ... [part i] [other parts]*
        // for as many times as part i exists
        for (i, pi) in (&parts).into_iter().enumerate() {
            // make [other parts]*
            let mut other_parts = vec![];
            for (j, pj) in (&parts).into_iter().enumerate() {
                if i != j {
                    other_parts.push(pj.fst.clone());
                }
            }
            let mut others = union_many(other_parts);
            closure(&mut others, closure::ClosureType::ClosureStar);

            let mut check_this_part = matches_empty();
            for _ in 0..pi.count {
                concat(&mut check_this_part, &others).unwrap(); // [others]*
                optimize_fst(&mut check_this_part);
                concat(&mut check_this_part, &pi.fst).unwrap(); // [part i]
                optimize_fst(&mut check_this_part);
            }
            concat(&mut check_this_part, &others).unwrap(); // add one last [others]*
            optimize_fst(&mut check_this_part);

            to_intersect.push(check_this_part);
        }

        for fst in &mut to_intersect {
            optimize_fst(fst);
            // let _ = fst.write_text(format!("generated.fst_{}.txt", k));
            // println!("{}", fst.num_states())
        }
        intersect_many(to_intersect)
    }
}

pub fn compile_expr(expr: Expr) -> VectorFst<TropicalWeight> {
    compile_quote_flag(expr, false)
}
