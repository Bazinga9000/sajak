use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{map, value},
    multi::{many1, many_m_n},
    sequence::delimited,
    IResult, Parser,
};

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Vec<char>),                 // abc...xyz012..789
    Any,                                // .
    Nonspace,                           // _
    Alpha,                              // A
    Consonant,                          // C
    Vowel,                              // V
    Digit,                              // #
    OptionalSpace,                      // -
    Word,                               // W
    Element,                            // P
    USState,                            // S
    OneOf(Vec<char>),                   // [abc...]
    NoneOf(Vec<char>),                  // [^abc..]
    Or(Box<Expr>, Box<Expr>),           // Expr | Expr
    And(Box<Expr>, Box<Expr>),          // Expr & Expr
    Star(Box<Expr>),                    // Expr*
    Question(Box<Expr>),                // Expr?
    Plus(Box<Expr>),                    // Expr+
    Reverse(Box<Expr>),                 // Expr~
    NCopies(Box<Expr>, u64),            // {n}
    Range(Box<Expr>, u64, Option<u64>), // {n,m}
    Quote(Box<Expr>),                   // [[Expr]]
    Anagram(Vec<Expr>),                 // <ExprExprExpr>
    Sequence(Vec<Expr>),                // ExprExpr
}

fn parse_legal_chars(input: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyz0123456789 ").parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expr> {
    map(parse_legal_chars, |cs| Expr::Literal(vec![cs])).parse(input)
}

fn parse_constant(input: &str) -> IResult<&str, Expr> {
    alt((
        value(Expr::Any, char('.')),
        value(Expr::Nonspace, char('_')),
        value(Expr::Alpha, char('A')),
        value(Expr::Consonant, char('C')),
        value(Expr::Vowel, char('V')),
        value(Expr::Digit, char('#')),
        value(Expr::OptionalSpace, char('-')),
        value(Expr::Word, char('W')),
        value(Expr::Element, char('P')),
        value(Expr::USState, char('S')),
    ))
    .parse(input)
}

fn parse_parenthetical(input: &str) -> IResult<&str, Expr> {
    delimited(char('('), parse_expr_raw, char(')')).parse(input)
}

fn parse_anagram(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(char('<'), parse_expr_raw, char('>')),
        |e| match e {
            Expr::Sequence(v) => Expr::Anagram(v),
            _ => Expr::Anagram(vec![e]),
        },
    )
    .parse(input)
}

fn parse_quoted(input: &str) -> IResult<&str, Expr> {
    map(delimited(tag("[["), parse_expr_raw, tag("]]")), |e| {
        Expr::Quote(Box::new(e))
    })
    .parse(input)
}

fn parse_one_of(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(char('['), many1(parse_legal_chars), char(']')),
        |v| Expr::OneOf(v),
    )
    .parse(input)
}

fn parse_none_of(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(tag("[^"), many1(parse_legal_chars), char(']')),
        |v| Expr::NoneOf(v),
    )
    .parse(input)
}

fn parse_nullary_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_parenthetical,
        parse_quoted,
        parse_anagram,
        parse_one_of,
        parse_none_of,
        parse_constant,
        parse_literal,
    ))
    .parse(input)
}

fn parse_unary_ops(input: &str) -> IResult<&str, Expr> {
    let pure_unary_op = map((parse_nullary_expr, one_of("*?+~")), |(e, c)| match c {
        '*' => Expr::Star(Box::new(e)),
        '?' => Expr::Question(Box::new(e)),
        '+' => Expr::Plus(Box::new(e)),
        '~' => Expr::Reverse(Box::new(e)),
        _ => unreachable!(),
    });

    let digit = nom::character::complete::u64;
    let n_copies_op = delimited(char('{'), digit, char('}'));
    let n_copies = map((parse_nullary_expr, n_copies_op), |(e, n)| {
        Expr::NCopies(Box::new(e), n)
    });

    let optional_digit = many_m_n(0, 1, digit).map(|v| v.into_iter().nth(0));
    let range_op = delimited(char('{'), (digit, char(','), optional_digit), char('}'));
    let range = map((parse_nullary_expr, range_op), |(e, (mn, _, mx))| {
        Expr::Range(Box::new(e), mn, mx)
    });

    alt((pure_unary_op, n_copies, range, parse_nullary_expr)).parse(input)
}

fn parse_concatenation(input: &str) -> IResult<&str, Expr> {
    map(many1(parse_unary_ops), |v| {
        if v.len() == 1 {
            v[0].clone()
        } else {
            Expr::Sequence(v)
        }
    })
    .parse(input)
}

fn parse_conjunction(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            (parse_concatenation, char('&'), parse_conjunction),
            |(e1, _, e2)| Expr::And(Box::new(e1), Box::new(e2)),
        ),
        parse_concatenation,
    ))
    .parse(input)
}

fn parse_alternative(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            (parse_conjunction, char('|'), parse_alternative),
            |(e1, _, e2)| Expr::Or(Box::new(e1), Box::new(e2)),
        ),
        parse_conjunction,
    ))
    .parse(input)
}

fn parse_expr_raw(input: &str) -> IResult<&str, Expr> {
    parse_alternative(input)
}

fn collapse_literals(expr: Expr) -> Expr {
    let c = |e: Box<Expr>| Box::new(collapse_literals(*e));
    match expr {
        Expr::Literal(_) => expr,
        Expr::Any => expr,
        Expr::Nonspace => expr,
        Expr::Alpha => expr,
        Expr::Consonant => expr,
        Expr::Vowel => expr,
        Expr::Digit => expr,
        Expr::OptionalSpace => expr,
        Expr::Word => expr,
        Expr::Element => expr,
        Expr::USState => expr,
        Expr::OneOf(_) => expr,
        Expr::NoneOf(_) => expr,
        Expr::Or(a, b) => Expr::Or(c(a), c(b)),
        Expr::And(a, b) => Expr::And(c(a), c(b)),
        Expr::Star(e) => Expr::Star(c(e)),
        Expr::Question(e) => Expr::Question(c(e)),
        Expr::Plus(e) => Expr::Plus(c(e)),
        Expr::Reverse(e) => Expr::Reverse(c(e)),
        Expr::NCopies(e, n) => Expr::NCopies(c(e), n),
        Expr::Range(e, mn, mx) => Expr::Range(c(e), mn, mx),
        Expr::Quote(e) => Expr::Quote(c(e)),
        Expr::Anagram(es) => Expr::Anagram(es.into_iter().map(collapse_literals).collect()),
        Expr::Sequence(vec) => {
            let mut new_sequence = vec![];
            let mut literal_stack = vec![];

            for e in vec.into_iter().map(collapse_literals) {
                match e {
                    Expr::Literal(cs) => literal_stack.extend(cs),
                    e => {
                        if !literal_stack.is_empty() {
                            new_sequence.push(Expr::Literal(literal_stack.drain(..).collect()))
                        }
                        new_sequence.push(e)
                    }
                }
            }
            if !literal_stack.is_empty() {
                new_sequence.push(Expr::Literal(literal_stack.drain(..).collect()))
            }

            if new_sequence.len() == 1 {
                new_sequence.iter().nth(0).unwrap().clone()
            } else {
                Expr::Sequence(new_sequence)
            }
        }
    }
}

pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    map(parse_expr_raw, collapse_literals).parse(input)
}
