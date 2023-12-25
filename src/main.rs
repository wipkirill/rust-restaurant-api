use anyhow::Result;
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::signal;

mod config;
mod domain;
use config::Opt;
mod client;

mod repository;
use repository::inmemory::InMemoryRepository;
use repository::sqlite::SqliteRepository;
use repository::Repository;

mod api;
use api::handle;
use api::server::serve;

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    guard
}
#[tokio::main]
async fn app(args: Opt) -> Result<()> {
    let _guard = init_logging();

    let addr = std::net::SocketAddr::new(args.address.parse()?, args.port);

    let context: Arc<dyn Repository> = build_repo(Option::Some("restaurant.sqlite"));
    let s_ctx = context.clone();
    tokio::spawn(async move { serve(addr, s_ctx, handle).await });

    let running = Arc::new(AtomicBool::new(true));
    let mut handles = vec![];

    let base_url = format!("http://{}:{}", &args.address, args.port);
    // Spawn 10 threads with client_fn
    for id in 1..=args.num_clients {
        let c = running.clone();
        let url = base_url.clone();
        let handle = thread::spawn(move || {
            client::client_main(
                id,
                &url,
                1000,
                c.clone(),
            );
        });

        handles.push(handle);
    }

    match signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Shutting down...");
            running.store(false, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(3000));
            //context.shutdown();
        }
        Err(err) => {
            tracing::info!("Unable to listen for shutdown signal: {}", err);
        }
    }
    tracing::info!("Shutted down");
    Ok(())
}

fn build_repo(sqlite_value: Option<&str>) -> Arc<dyn Repository> {
    if let Some(path) = sqlite_value {
        match SqliteRepository::try_new(path) {
            Ok(repo) => return Arc::new(repo),
            _ => panic!("Error while creating sqlite repo, using in-memory repo"),
        }
    }

    Arc::new(InMemoryRepository::new())
}

fn main() -> Result<()> {
    let args = Opt::parse();
    app(args)?;
    Ok(())
}
