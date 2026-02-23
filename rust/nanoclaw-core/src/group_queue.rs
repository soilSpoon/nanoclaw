use std::collections::{HashSet, VecDeque};

#[derive(Debug, Default)]
pub struct GroupQueue {
    queued: VecDeque<String>,
    queued_set: HashSet<String>,
    active: HashSet<String>,
    max_concurrency: usize,
}

impl GroupQueue {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            queued: VecDeque::new(),
            queued_set: HashSet::new(),
            active: HashSet::new(),
            max_concurrency,
        }
    }

    pub fn enqueue(&mut self, group_id: impl Into<String>) {
        let id = group_id.into();
        if self.active.contains(&id) {
            return;
        }
        if self.queued_set.insert(id.clone()) {
            self.queued.push_back(id);
        }
    }

    pub fn next(&mut self) -> Option<String> {
        if self.active.len() >= self.max_concurrency {
            return None;
        }
        let id = self.queued.pop_front()?;
        self.queued_set.remove(&id);
        self.active.insert(id.clone());
        Some(id)
    }

    pub fn complete(&mut self, group_id: &str) {
        self.active.remove(group_id);
    }

    pub fn is_active(&self, group_id: &str) -> bool {
        self.active.contains(group_id)
    }
}

#[cfg(test)]
mod tests {
    use super::GroupQueue;

    #[test]
    fn enforces_global_concurrency() {
        let mut q = GroupQueue::new(1);
        q.enqueue("g1");
        q.enqueue("g2");

        assert_eq!(q.next().as_deref(), Some("g1"));
        assert_eq!(q.next(), None);

        q.complete("g1");
        assert_eq!(q.next().as_deref(), Some("g2"));
    }

    #[test]
    fn blocks_duplicate_when_active_or_queued() {
        let mut q = GroupQueue::new(2);
        q.enqueue("g1");
        q.enqueue("g1");
        assert_eq!(q.next().as_deref(), Some("g1"));
        q.enqueue("g1");
        assert_eq!(q.next(), None);
    }
}
