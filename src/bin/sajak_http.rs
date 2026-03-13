use actix_web::{middleware, web, App, HttpServer};
use sajak::{
    corpus::trie::CorpusTrie,
    frontends::{default_trie_path, load_default_tree},
    http::{health::health, query::sajak_query},
};
use std::env;
use std::sync::LazyLock;

static TRIE: LazyLock<CorpusTrie> = LazyLock::new(|| {
    load_default_tree().expect(&format!(
        "The default corpus trie is either nonexistent or invalid. Please ensure you have a valid Sajak trie at {}",
        default_trie_path().display()
    ))
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "1983".to_string());
    let bind_addr = format!("127.0.0.1:{}", port);

    log::info!("Loading corpus ferom {}", default_trie_path().display());
    log::info!("Corpus loaded ({} entries)", TRIE.num_entries); // this also inits the lazylock
    log::info!("Starting server at {}", bind_addr);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health))
            .route("/query", web::get().to(|i| sajak_query(&TRIE, i, None)))
    })
    .bind(&bind_addr)?
    .workers(num_cpus::get())
    .run()
    .await
}
