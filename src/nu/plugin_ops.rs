use crate::{corpus::trie::CorpusTrie, expr::Expr};
use nom::Parser;
use nu_protocol::{LabeledError, Span, Value};
use std::path::PathBuf;

pub fn parse_expr_nu(query_value: &Value) -> Result<Expr, LabeledError> {
    let query = query_value.as_str()?;
    match nom::combinator::all_consuming(crate::expr::parse_expr).parse(query) {
        Err(nom::Err::Error(nom::error::Error { input: e, .. })) => {
            return Err(LabeledError::new("Query Parse Error")
                .with_label(format!("Parse error at \"{}\"", e), query_value.span()));
        }

        Ok((_, expr)) => Ok(expr),
        _ => unreachable!(),
    }
}

pub fn load_trie_from_file(path: PathBuf, error_span: Span) -> Result<CorpusTrie, LabeledError> {
    let raw = std::fs::read(&path).map_err(|_| {
        LabeledError::new("File read error").with_label(
            format!("Could not read trie file at {:#?}", path),
            error_span,
        )
    })?;

    match CorpusTrie::from_bytes(&raw) {
        Some(t) => Ok(t),
        None => Err(LabeledError::new("Trie deserialization error").with_label(
            format!("File {:#?} is not a valid Sajak corpus.", path),
            error_span,
        )),
    }
}
