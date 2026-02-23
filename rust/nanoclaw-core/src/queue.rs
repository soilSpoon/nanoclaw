use std::collections::{HashSet, VecDeque};

#[derive(Debug, Default)]
pub struct GroupQueue {
    queued: VecDeque<String>,
    queued_set: HashSet<String>,
}

impl GroupQueue {
    pub fn enqueue(&mut self, group_id: impl Into<String>) {
        let id = group_id.into();
        if self.queued_set.insert(id.clone()) {
            self.queued.push_back(id);
        }
    }

    pub fn dequeue(&mut self) -> Option<String> {
        let id = self.queued.pop_front()?;
        self.queued_set.remove(&id);
        Some(id)
    }

    pub fn len(&self) -> usize {
        self.queued.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queued.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::GroupQueue;

    #[test]
    fn preserves_fifo() {
        let mut q = GroupQueue::default();
        q.enqueue("g1");
        q.enqueue("g2");
        assert_eq!(q.dequeue().as_deref(), Some("g1"));
        assert_eq!(q.dequeue().as_deref(), Some("g2"));
    }

    #[test]
    fn deduplicates_inflight_group() {
        let mut q = GroupQueue::default();
        q.enqueue("g1");
        q.enqueue("g1");
        assert_eq!(q.len(), 1);
        assert_eq!(q.dequeue().as_deref(), Some("g1"));
        assert!(q.is_empty());
    }
}
