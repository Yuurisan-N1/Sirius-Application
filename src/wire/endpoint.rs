pub const REGISTRATION_CONFIG: &str = "/auth/registration-config";
pub const REGISTER: &str = "/auth/register";
pub const LOGIN: &str = "/auth/login";

pub const ME: &str = "/users/me";
pub const INVITER: &str = "/users/me/inviter";

pub const TASKS: &str = "/tasks";
pub const TASKS_SUMMARY: &str = "/tasks/summary";

pub const MINING_START: &str = "/mining/tasks/start";
pub const MINING_STATUS: &str = "/mining/tasks/status";

pub const REGION_CHECK: &str = "/region/check";

pub fn participation(task_id: u64) -> String {
    format!("/tasks/{}/participations", task_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_path_starts_with_a_slash() {
        for path in [
            REGISTRATION_CONFIG,
            REGISTER,
            LOGIN,
            ME,
            INVITER,
            TASKS,
            TASKS_SUMMARY,
            MINING_START,
            MINING_STATUS,
            REGION_CHECK,
        ] {
            assert!(path.starts_with('/'));
        }
    }

    #[test]
    fn the_participation_path_is_built_from_the_id() {
        assert_eq!(participation(4), "/tasks/4/participations");
    }
}
