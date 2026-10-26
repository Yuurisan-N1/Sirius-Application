use std::time::Duration;

use serde_json::json;

use crate::net::client::Link;
use crate::support::consts::app::{SETTLE_MILLIS, SETTLE_POLLS};
use crate::support::error::AppResult;
use crate::wire::endpoint::{participation, TASKS, TASKS_SUMMARY};
use crate::wire::shape::{parse, Claim, Task, TaskBoard, TaskSummary};

pub async fn board(link: &Link) -> AppResult<Vec<Task>> {
    let data = link.get(TASKS).await?;
    let mut board: TaskBoard = parse(data, "task list")?;
    board.tasks.sort_by_key(|row| row.sort_order);
    Ok(board.tasks)
}

pub async fn claim(link: &Link, task_id: u64) -> AppResult<Claim> {
    let data = link
        .post(&participation(task_id), &json!({}))
        .await?;
    parse(data, "task claim")
}

pub async fn await_settle(link: &Link, task_id: u64) -> AppResult<bool> {
    for _ in 0..SETTLE_POLLS {
        tokio::time::sleep(Duration::from_millis(SETTLE_MILLIS)).await;
        match board(link).await {
            Ok(rows) => {
                if rows
                    .iter()
                    .any(|row| row.task_id == task_id && row.settled())
                {
                    return Ok(true);
                }
            }
            Err(_) => continue,
        }
    }
    Ok(false)
}

pub async fn summary(link: &Link) -> AppResult<TaskSummary> {
    let data = link.get(TASKS_SUMMARY).await?;
    parse(data, "task summary")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_board_keeps_the_server_order() {
        let text = r#"{"tasks":[{"taskId":2,"sortOrder":20},{"taskId":1,"sortOrder":10}]}"#;
        let mut board: TaskBoard = serde_json::from_str(text).unwrap();
        board.tasks.sort_by_key(|row| row.sort_order);
        assert_eq!(board.tasks[0].task_id, 1);
    }

    #[test]
    fn the_participation_path_is_built_from_the_id() {
        assert_eq!(participation(4), "/tasks/4/participations");
    }
}
