use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncEvent {
    BatchSent { changes: usize, bytes: usize },
    BatchReceived { changes: usize, bytes: usize },
    ChangeApplied,
    DuplicateIgnored,
    ChangeRejected,
    Retry,
    Conflict,
    QueueDepth { depth: usize },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMetrics {
    pub batches_sent: u64,
    pub batches_received: u64,
    pub changes_applied: u64,
    pub duplicates_ignored: u64,
    pub changes_rejected: u64,
    pub retries: u64,
    pub conflicts: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub queue_depth: u64,
}

impl SyncMetrics {
    pub fn observe(&mut self, event: &SyncEvent) {
        match event {
            SyncEvent::BatchSent { bytes, .. } => {
                self.batches_sent += 1;
                self.bytes_sent = self.bytes_sent.saturating_add(*bytes as u64);
            }
            SyncEvent::BatchReceived { bytes, .. } => {
                self.batches_received += 1;
                self.bytes_received = self.bytes_received.saturating_add(*bytes as u64);
            }
            SyncEvent::ChangeApplied => self.changes_applied += 1,
            SyncEvent::DuplicateIgnored => self.duplicates_ignored += 1,
            SyncEvent::ChangeRejected => self.changes_rejected += 1,
            SyncEvent::Retry => self.retries += 1,
            SyncEvent::Conflict => self.conflicts += 1,
            SyncEvent::QueueDepth { depth } => self.queue_depth = *depth as u64,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.batches_sent = self.batches_sent.saturating_add(other.batches_sent);
        self.batches_received = self.batches_received.saturating_add(other.batches_received);
        self.changes_applied = self.changes_applied.saturating_add(other.changes_applied);
        self.duplicates_ignored = self.duplicates_ignored.saturating_add(other.duplicates_ignored);
        self.changes_rejected = self.changes_rejected.saturating_add(other.changes_rejected);
        self.retries = self.retries.saturating_add(other.retries);
        self.conflicts = self.conflicts.saturating_add(other.conflicts);
        self.bytes_sent = self.bytes_sent.saturating_add(other.bytes_sent);
        self.bytes_received = self.bytes_received.saturating_add(other.bytes_received);
        self.queue_depth = other.queue_depth;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_accumulate_deterministically() {
        let mut metrics = SyncMetrics::default();
        metrics.observe(&SyncEvent::BatchSent { changes: 2, bytes: 100 });
        metrics.observe(&SyncEvent::BatchReceived { changes: 2, bytes: 120 });
        metrics.observe(&SyncEvent::ChangeApplied);
        metrics.observe(&SyncEvent::DuplicateIgnored);
        metrics.observe(&SyncEvent::ChangeRejected);
        metrics.observe(&SyncEvent::Retry);
        metrics.observe(&SyncEvent::Conflict);
        metrics.observe(&SyncEvent::QueueDepth { depth: 7 });

        assert_eq!(metrics.batches_sent, 1);
        assert_eq!(metrics.batches_received, 1);
        assert_eq!(metrics.changes_applied, 1);
        assert_eq!(metrics.duplicates_ignored, 1);
        assert_eq!(metrics.changes_rejected, 1);
        assert_eq!(metrics.retries, 1);
        assert_eq!(metrics.conflicts, 1);
        assert_eq!(metrics.bytes_sent, 100);
        assert_eq!(metrics.bytes_received, 120);
        assert_eq!(metrics.queue_depth, 7);
    }

    #[test]
    fn replicas_can_merge_counter_snapshots() {
        let mut a = SyncMetrics::default();
        a.observe(&SyncEvent::BatchSent { changes: 1, bytes: 10 });
        a.observe(&SyncEvent::QueueDepth { depth: 2 });

        let mut b = SyncMetrics::default();
        b.observe(&SyncEvent::Retry);
        b.observe(&SyncEvent::Conflict);
        b.observe(&SyncEvent::QueueDepth { depth: 5 });

        a.merge(&b);
        assert_eq!(a.batches_sent, 1);
        assert_eq!(a.retries, 1);
        assert_eq!(a.conflicts, 1);
        assert_eq!(a.bytes_sent, 10);
        assert_eq!(a.queue_depth, 5);
    }
}
