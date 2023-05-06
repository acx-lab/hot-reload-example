use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::Extension,
    routing::get,
    Router,
};
use crossbeam::atomic::AtomicCell;

#[derive(Clone)]
struct Config {
    value: Arc<AtomicCell<Option<String>>>,
}

impl Config {
    fn new() -> Self {
        Self {
            value: Arc::new(AtomicCell::new(None)),
        }
    }

    async fn load(&self, path: &str) -> Result<(), std::io::Error> {
        let content = tokio::fs::read_to_string(path).await?;
        self.value.store(Some(content));

        Ok(())
    }

    fn get(&self) -> Option<String> {
        let t = self.value.take();
        self.value.store(t.clone());

        t
    }
}

// idea
// implement a struct that allows for concurrent lock-free reads
// the struct should be able to update its value while incurring no read-time loss
#[tokio::main]
async fn main() {
    let config = Config::new();
    config.load("config.json").await.unwrap();

    let config_update = config.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            config_update.load("config.json").await.unwrap();
        }
    });
    let app = Router::new()
        .route("/", get(root))
        .layer(Extension(config));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root(Extension(config): Extension<Config>) -> String {
    config.get().unwrap_or_default()
}
