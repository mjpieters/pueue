#![allow(dead_code)]
use anyhow::Result;
use tokio::io::{self, AsyncWriteExt};

pub use pueue_lib::state::PUEUE_DEFAULT_GROUP;

mod client;
mod daemon;
mod env;
mod group;
mod log;
mod message;
mod network;
mod state;
mod task;
mod wait;

pub use self::log::*;
pub use client::*;
pub use daemon::*;
pub use env::*;
pub use group::*;
pub use message::*;
pub use network::*;
pub use state::*;
pub use task::*;
pub use wait::*;

/// A helper function to sleep for ms time.
/// Only used to avoid the biolerplate of importing the same stuff all over the place.
pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// A small helper function, which instantly writes the given string to stdout with a newline.
/// Useful for debugging async tests.
pub async fn async_println(out: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .write_all(out.as_bytes())
        .await
        .expect("Failed to write to stdout.");

    stdout
        .write_all("\n".as_bytes())
        .await
        .expect("Failed to write to stdout.");
    stdout.flush().await?;

    Ok(())
}
