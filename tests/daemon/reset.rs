use anyhow::{Context, Result};
use pueue_lib::network::message::*;

use crate::fixtures::*;
use crate::helper::*;

/// A reset command kills all tasks and forces a clean slate.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_reset() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Start a long running task and make sure it's started
    add_task(shared, "ls", false).await?;
    add_task(shared, "failed", false).await?;
    add_task(shared, "sleep 60", false).await?;
    add_task(shared, "ls", false).await?;
    wait_for_task_condition(shared, 1, |task| task.is_running()).await?;

    // Reset the daemon
    send_message(shared, ResetMessage { children: true })
        .await
        .context("Failed to send Start tasks message")?;

    // Reseting is asynchronous, wait for the first task to disappear.
    wait_for_task_absence(shared, 0).await?;

    // All tasks should have been removed.
    let state = get_state(shared).await?;
    assert!(state.tasks.is_empty(),);

    Ok(())
}
