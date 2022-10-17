/// This module contains helper functions to work with Pueue's asynchronous nature during tests.
/// As the daemon often needs some time to process requests by the client internally, we cannot
/// check whether the requested actions have been taken right away.
///
/// Using continuous lookups, we can allow long waiting times, while still having fast tests if
/// things don't take that long.
use anyhow::{bail, Result};

use pueue_lib::settings::Shared;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::{Task, TaskStatus};

use super::{get_state, sleep_ms};

/// This is a small helper function, which checks in very short intervals, whether a task showed up
/// in the daemon or not.
pub async fn wait_for_task(shared: &Shared, task_id: usize) -> Result<Task> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        if !state.tasks.contains_key(&task_id) {
            current_try += 1;
            sleep_ms(50);
            continue;
        }

        return Ok(state.tasks.get(&task_id).unwrap().clone());
    }

    bail!("Task {} didn't show up in about 1 second.", task_id)
}

/// This is a small helper function, which checks in very short intervals, whether a task changed
/// it's state or not.
pub async fn wait_for_status_change(
    shared: &Shared,
    task_id: usize,
    original_status: TaskStatus,
) -> Result<Task> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        match state.tasks.get(&task_id) {
            Some(task) => {
                // The status changed. We can give our ok!
                if task.status != original_status {
                    return Ok(task.clone());
                }

                // The status didn't change. Try again.
                current_try += 1;
                sleep_ms(50);
                continue;
            }
            None => {
                bail!("Couldn't find task {task_id} while waiting for status change")
            }
        }
    }

    bail!("Task {task_id} didn't change state in about 1 second.")
}

/// This is a small helper function, which checks in very short intervals, whether a task fulfills
/// a certain criteria. This is necessary to prevent long or potentially flaky timeouts in our tests.
pub async fn wait_for_task_condition<F>(
    shared: &Shared,
    task_id: usize,
    condition: F,
) -> Result<Task>
where
    F: Fn(&Task) -> bool,
{
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        match state.tasks.get(&task_id) {
            Some(task) => {
                // Check if the condition is met.
                // If it isn't, continue
                if condition(task) {
                    return Ok(task.clone());
                }

                // The status didn't change to target. Try again.
                current_try += 1;
                sleep_ms(50);
                continue;
            }
            None => {
                bail!("Couldn't find task {task_id} while waiting for condition")
            }
        }
    }
    bail!("Task {task_id} didn't fulfill condition after about 1 second.")
}

/// This is a small helper function, which checks in very short intervals, whether a task has been
/// deleted. This is necessary, as task deletion is asynchronous task.
pub async fn wait_for_task_absence(shared: &Shared, task_id: usize) -> Result<()> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        if state.tasks.contains_key(&task_id) {
            current_try += 1;
            sleep_ms(50);
            continue;
        }

        return Ok(());
    }

    bail!("Task {task_id} hasn't been removed after about 1 second.")
}

/// This is a small helper function, which checks in very short intervals, whether a group has been
/// initialized. This is necessary, as group creation became an asynchronous task.
pub async fn wait_for_group(shared: &Shared, group: &str) -> Result<()> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        if !state.groups.contains_key(group) {
            current_try += 1;
            sleep_ms(50);
            continue;
        }

        return Ok(());
    }

    bail!("Group {group} didn't show up in about 1 second.")
}

/// This is a small helper function, which checks in very short intervals, whether a group has been
/// deleted. This is necessary, as group deletion became an asynchronous task.
pub async fn wait_for_group_absence(shared: &Shared, group: &str) -> Result<()> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        if state.groups.contains_key(group) {
            current_try += 1;
            sleep_ms(50);
            continue;
        }

        return Ok(());
    }

    bail!("Group {group} hasn't been removed after about 1 second.")
}

/// Waits for a status on a specific group.
pub async fn wait_for_group_status(
    shared: &Shared,
    group: &str,
    expected_status: GroupStatus,
) -> Result<()> {
    // Give the daemon about 1 second to change group status.
    let tries = 20;
    let mut current_try = 0;

    while current_try < tries {
        let state = get_state(shared).await?;
        match state.groups.get(group) {
            Some(group) => {
                if group.status == expected_status {
                    return Ok(());
                }

                // The status didn't change to the expected status. Try again.
                current_try += 1;
                sleep_ms(50);
                continue;
            }
            None => {
                bail!("Couldn't find group {group} while waiting for status change")
            }
        }
    }

    bail!("Group {group} didn't change to state {expected_status:?} after about 1 second",);
}
