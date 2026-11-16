//! Persistent SDK transport for the isolated ordinary-scene authority checks.

use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use adventuresim_stdb_client::spacetimedb_sdk::DbContext;
use adventuresim_stdb_client::{DbConnection, create_tactical_server_for_request};
use clap::Parser;

const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(15);
const ACCEPTANCE_PEER_LIFETIME: Duration = Duration::from_secs(240);
const ACCEPTANCE_SERVER_ADDRESS: &str = "127.0.0.1:1";

#[derive(Parser)]
struct Args {
    #[arg(long)]
    url: String,
    #[arg(long)]
    database: String,
    #[arg(long)]
    instance_id: String,
    #[arg(long, env = "SPACETIMEDB_TOKEN", hide_env_values = true)]
    token: String,
    #[arg(long, env = "ADVENTURESIM_TACTICAL_CLAIM", hide_env_values = true)]
    claim: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let (registration, registered) = mpsc::channel();
    let conn = DbConnection::builder()
        .with_uri(args.url)
        .with_database_name(args.database)
        .with_token(Some(args.token))
        .on_connect(move |ctx, _, _| {
            let failed = registration.clone();
            let completed = registration.clone();
            if let Err(error) = ctx.reducers.create_tactical_server_for_request_then(
                args.instance_id.clone(),
                args.claim.clone(),
                ACCEPTANCE_SERVER_ADDRESS.into(),
                String::new(),
                move |_, result| {
                    let result = result.unwrap_or_else(|error| Err(error.to_string()));
                    let _ = completed.send(result);
                },
            ) {
                let _ = failed.send(Err(error.to_string()));
            }
        })
        .build()?;
    let start = Instant::now();
    loop {
        conn.frame_tick()?;
        match registered.try_recv() {
            Ok(result) => {
                result.map_err(io::Error::other)?;
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(io::Error::other("Registration callback disappeared").into());
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if start.elapsed() >= REGISTRATION_TIMEOUT {
            return Err(io::Error::other("Scene registration timed out").into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    println!("READY");
    let (stop, stopped) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
        let _ = stop.send(());
    });
    while stopped.try_recv().is_err() && start.elapsed() < ACCEPTANCE_PEER_LIFETIME {
        conn.frame_tick()?;
        std::thread::sleep(Duration::from_millis(10));
    }
    conn.disconnect()?;
    while conn.advance_one_message_blocking().is_ok() {}
    Ok(())
}
