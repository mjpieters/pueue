use std::convert::TryInto;

use anyhow::{Context, Result};

use crate::fixtures::*;
use crate::helper::*;

#[tokio::test]
/// Spin up the daemon and send a SIGTERM shortly afterwards.
/// This should trigger the graceful shutdown and kill the process.
async fn test_ctrlc() -> Result<()> {
    let (settings, _tempdir) = daemon_base_setup()?;
    let mut child = standalone_daemon(&settings.shared).await?;

    use nix::sys::signal::{kill, Signal};
    // Send SIGTERM signal to process via nix
    let nix_pid = nix::unistd::Pid::from_raw(child.id() as i32);
    kill(nix_pid, Signal::SIGTERM).context("Failed to send SIGTERM to pid")?;

    // Sleep for 500ms and give the daemon time to shut down
    sleep_ms(500).await;

    let result = child.try_wait();
    assert!(matches!(result, Ok(Some(_))));
    let code = result.unwrap().unwrap();
    assert!(matches!(code.code(), Some(1)));

    Ok(())
}

#[tokio::test]
/// Spin up the daemon and send a graceful shutdown message afterwards.
/// The daemon should shutdown normally and exit with a 0.
async fn test_graceful_shutdown() -> Result<()> {
    let (settings, _tempdir) = daemon_base_setup()?;
    let mut child = standalone_daemon(&settings.shared).await?;

    // Kill the daemon gracefully and wait for it to shut down.
    assert_success(shutdown_daemon(&settings.shared).await?);
    wait_for_shutdown(child.id().try_into()?).await?;

    // Sleep for 500ms and give the daemon time to shut down
    sleep_ms(500).await;

    let result = child.try_wait();
    assert!(matches!(result, Ok(Some(_))));
    let code = result.unwrap().unwrap();
    assert!(matches!(code.code(), Some(0)));

    Ok(())
}
