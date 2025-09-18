use nu_protocol::{LabeledError, Record, Value};

use crate::corpus::search::SearchResult;

fn result_to_row(result: SearchResult, tag: &nu_protocol::Span) -> [(&'_ str, Value); 6] {
    [
        ("result", Value::string(result.result, *tag)),
        ("score", Value::float(result.score, *tag)),
        ("length", Value::int(result.length as i64, *tag)),
        ("letters", Value::int(result.length_nospace as i64, *tag)),
        ("words", Value::int(result.num_words as i64, *tag)),
        ("scrabble", Value::int(result.scrabble as i64, *tag)),
    ]
}

pub fn tablify_results(
    results: Vec<SearchResult>,
    tag: &nu_protocol::Span,
) -> Result<Value, LabeledError> {
    let vec = results
        .into_iter()
        .map(|r| {
            let row = result_to_row(r, tag);
            let record = row
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect::<Record>();
            Value::record(record, *tag)
        })
        .collect();
    Ok(Value::list(vec, *tag))
}
