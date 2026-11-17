//! Persistent SDK transport for the isolated ordinary-scene authority checks.

use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use adventuresim_stdb_client::spacetimedb_sdk::DbContext;
use adventuresim_stdb_client::{DbConnection, create_tactical_server_for_request};
use clap::Parser;

#[path = "settlement_authority_peer/protocol.rs"]
mod protocol;

const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(15);
const ACCEPTANCE_PEER_LIFETIME: Duration = Duration::from_secs(240);
const ACCEPTANCE_SERVER_ADDRESS: &str = "127.0.0.1:1";
const PEER_POLL_INTERVAL: Duration = Duration::from_millis(10);

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
    let conn = Args::parse().connect()?;
    println!("READY");
    let requests = protocol::input();
    let start = Instant::now();
    while start.elapsed() < ACCEPTANCE_PEER_LIFETIME {
        conn.frame_tick()?;
        while let Ok(command) = requests.try_recv() {
            if matches!(command.dispatch(&conn), protocol::PeerControl::Stop) {
                conn.disconnect()?;
                while conn.advance_one_message_blocking().is_ok() {}
                return Ok(());
            }
        }
        std::thread::sleep(PEER_POLL_INTERVAL);
    }
    conn.disconnect()?;
    while conn.advance_one_message_blocking().is_ok() {}
    Ok(())
}

impl Args {
    fn connect(self) -> Result<DbConnection, Box<dyn std::error::Error>> {
        let (registration, registered) = mpsc::channel();
        let conn = DbConnection::builder()
            .with_uri(self.url)
            .with_database_name(self.database)
            .with_token(Some(self.token))
            .on_connect(move |ctx, _, _| {
                let failed = registration.clone();
                let completed = registration.clone();
                if let Err(error) = ctx.reducers.create_tactical_server_for_request_then(
                    self.instance_id.clone(),
                    self.claim.clone(),
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
        await_ready(&conn, registered)?;
        let (applied, ready) = mpsc::channel();
        let failed = applied.clone();
        conn.subscription_builder()
            .on_applied(move |_| {
                let _ = applied.send(Ok(()));
            })
            .on_error(move |_, error| {
                let _ = failed.send(Err(error.to_string()));
            })
            .subscribe(["SELECT * FROM connected_players"]);
        await_ready(&conn, ready)?;
        Ok(conn)
    }
}

fn await_ready(
    conn: &DbConnection,
    ready: mpsc::Receiver<Result<(), String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    loop {
        conn.frame_tick()?;
        match ready.try_recv() {
            Ok(result) => {
                result.map_err(io::Error::other)?;
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(io::Error::other("Peer readiness callback disappeared").into());
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if start.elapsed() >= REGISTRATION_TIMEOUT {
            return Err(io::Error::other("Scene peer readiness timed out").into());
        }
        std::thread::sleep(PEER_POLL_INTERVAL);
    }
    Ok(())
}
