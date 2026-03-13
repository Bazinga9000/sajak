use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::mpsc, time::Duration};
use std::thread;

use crate::{compile::compile_expr, corpus::trie::CorpusTrie, expr::Expr, http::error::SajakError};
use actix_web::{web, HttpResponse};
use nom::Parser;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SajakQuery {
    query: String,
    max_results: Option<u16>,
    max_nodes: Option<f32>,
    enable_loopbacks: Option<bool>,
}

fn parse_expr_http(query: &str) -> Result<Expr, SajakError> {
    match nom::combinator::all_consuming(crate::expr::parse_expr).parse(query) {
        Err(nom::Err::Error(nom::error::Error { input: e, .. })) => Err(SajakError::ParseError {
            input: e.to_string(),
        }),

        Ok((_, expr)) => Ok(expr),
        _ => unreachable!(),
    }
}

pub async fn sajak_query(
    trie: &CorpusTrie,
    item: web::Json<SajakQuery>,
    timeout: Option<Duration>
) -> Result<HttpResponse, SajakError> {
    log::info!("Running sajak query: {}", item.query);

    let timeout = timeout.unwrap_or(Duration::from_secs(30));
    let query = &item.query;
    let max_results = item.max_results.unwrap_or(10) as usize;
    let max_nodes = item.max_nodes.unwrap_or(4.0).min(10.0) * 1_000_000.0;
    let enable_loopbacks = item.enable_loopbacks.unwrap_or(true);

    if max_results == 0 {
        return Err(SajakError::MustBePositive {
            field: "Number of results".into(),
        });
    }

    if max_nodes <= 0.0 {
        return Err(SajakError::MustBePositive {
            field: "Number of nodes to search".into(),
        });
    }

    let expr = parse_expr_http(query)?;
    let fst = compile_expr(expr);
    let flag = AtomicBool::new(false);
    let (send, recv) = mpsc::channel();
    let results = thread::scope(|s| {
        let worker = s.spawn(|| {
            let res = trie.perform_search(fst, enable_loopbacks, Some(&flag), max_nodes.floor() as u64, max_results);
            send.send(res).unwrap_or(()); // Nothing ever happens.
            // Ignore all outputs, since either could be valid states. Ok is normal, Err could be the receiver hanging up due to timeout.
        });
        let res = match recv.recv_timeout(timeout) {
            Ok(res) => Ok(res),
            Err(_) => Err(SajakError::Timeout)
        };
        flag.store(true, Ordering::Relaxed);
        worker.join().unwrap_or(());
        res
    });

    results.map(|results| HttpResponse::Ok().json(results))
}
