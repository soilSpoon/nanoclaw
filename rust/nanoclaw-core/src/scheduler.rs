#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleType {
    IntervalMs,
    OnceIso,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub schedule_type: ScheduleType,
    pub schedule_value: String,
    pub next_run: Option<String>,
    pub status: String,
}

pub fn due_tasks<'a>(tasks: &'a [Task], now_iso: &str) -> Vec<&'a Task> {
    tasks.iter()
        .filter(|t| t.status == "active")
        .filter(|t| t.next_run.as_deref().map(|n| n <= now_iso).unwrap_or(false))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{due_tasks, ScheduleType, Task};

    #[test]
    fn selects_only_active_due_tasks() {
        let tasks = vec![
            Task {
                id: "1".into(),
                schedule_type: ScheduleType::OnceIso,
                schedule_value: "2026-01-01T00:00:00Z".into(),
                next_run: Some("2026-01-01T00:00:00Z".into()),
                status: "active".into(),
            },
            Task {
                id: "2".into(),
                schedule_type: ScheduleType::IntervalMs,
                schedule_value: "60000".into(),
                next_run: Some("2026-01-01T00:00:00Z".into()),
                status: "paused".into(),
            },
        ];

        let due = due_tasks(&tasks, "2026-01-01T00:00:00Z");
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, "1");
    }
}
