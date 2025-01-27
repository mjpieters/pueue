use std::collections::HashMap;
use std::env::vars;

use anyhow::{anyhow, Context, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::task::{Task, TaskStatus};

use crate::helper::*;

/// Adds a task to the test daemon.
pub async fn add_task(shared: &Shared, command: &str, start_immediately: bool) -> Result<Message> {
    let mut message = create_add_message(shared, command);
    message.start_immediately = start_immediately;

    send_message(shared, message)
        .await
        .context("Failed to to add task.")
}

/// Create a bare AddMessage for testing.
/// This is just here to minimize boilerplate code.
pub fn create_add_message(shared: &Shared, command: &str) -> AddMessage {
    AddMessage {
        command: command.into(),
        path: shared.pueue_directory(),
        envs: HashMap::from_iter(vars()),
        start_immediately: false,
        stashed: false,
        group: PUEUE_DEFAULT_GROUP.to_string(),
        enqueue_at: None,
        dependencies: Vec::new(),
        label: None,
        print_task_id: false,
    }
}

/// Adds a task to a specific group of the test daemon.
pub async fn add_task_to_group(shared: &Shared, command: &str, group: &str) -> Result<Message> {
    let mut message = create_add_message(shared, command);
    message.group = group.to_string();

    send_message(shared, message)
        .await
        .context("Failed to to add task to group.")
}

/// Mini wrapper around add_task, which creates a task that echos PUEUE's worker environment
/// variables to `stdout`.
pub async fn add_env_task(shared: &Shared, command: &str) -> Result<Message> {
    let command = format!(
        "echo WORKER_ID: $PUEUE_WORKER_ID; echo GROUP: $PUEUE_GROUP; {}",
        command
    );
    add_task(shared, &command, false).await
}

/// Just like [add_env_task], but the task get's added to specific group.
pub async fn add_env_task_to_group(shared: &Shared, command: &str, group: &str) -> Result<Message> {
    let command = format!("echo WORKER_ID: $PUEUE_WORKER_ID; echo GROUP: $PUEUE_GROUP; {command}");
    add_task_to_group(shared, &command, group).await
}

/// Helper to either continue the daemon or start specific tasks
pub async fn start_tasks(shared: &Shared, tasks: TaskSelection) -> Result<Message> {
    let message = StartMessage {
        tasks,
        children: false,
    };

    send_message(shared, message)
        .await
        .context("Failed to send Start tasks message")
}

/// Helper to pause the default group of the daemon
pub async fn pause_tasks(shared: &Shared, tasks: TaskSelection) -> Result<Message> {
    let message = PauseMessage {
        tasks,
        wait: false,
        children: false,
    };

    send_message(shared, message)
        .await
        .context("Failed to send Pause message")
}

/// Convenience wrapper around `get_state` if you only need a single task.
pub async fn get_task(shared: &Shared, task_id: usize) -> Result<Task> {
    let state = get_state(shared).await?;
    let task = state
        .tasks
        .get(&0)
        .ok_or_else(|| anyhow!("Couldn't find task {task_id}"))?;

    Ok(task.clone())
}

/// Convenience wrapper around `get_task` if you really only need the task's status.
pub async fn get_task_status(shared: &Shared, task_id: usize) -> Result<TaskStatus> {
    let task = get_task(shared, task_id).await?;
    Ok(task.status)
}
