//! The code here is referenced with [AppFlowy-Collab].
//! `wait_next_millis` make timestamp|sequenceis crdt/increased.
use std::{
    sync::atomic::{
        AtomicU64,
        Ordering::{Acquire, Release, SeqCst},
    },
    time::SystemTime,
};

pub const EPOCH: u64 = 1637806706000;
#[cfg(not(test))]
const SEQUENCE_BITS: u8 = 12;
#[cfg(test)]
const SEQUENCE_BITS: u8 = 1;
const TIMESTAMP_SHIFT: u8 = SEQUENCE_BITS;
const SEQUENCE_MASK: u64 = (1 << SEQUENCE_BITS) - 1;

pub type NID = u64;
pub type SIDGEN = AtomicU64;
pub type SID = u64;

/// Represents a sequential number generator.
#[derive(Default)]
pub struct Seq(SIDGEN);

impl Seq {
    /// Creates a new `Seq` instance with an initial value of 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use idable::Seq;
    ///
    /// let mut seq = Seq::new();
    /// ```
    pub fn new() -> Seq {
        Seq(SIDGEN::new(0))
    }

    /// Generates the next sequential ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use idable::Seq;
    ///
    /// let mut seq = Seq::new();
    /// let next_id = seq.next_id();
    /// ```
    pub fn next_id(&mut self) -> SID {
        self.0.fetch_add(1, SeqCst)
    }

    /// Resets the sequential number to its initial value of 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use idable::Seq;
    ///
    /// let mut seq = Seq::new();
    /// seq.reset();
    /// ```
    pub fn reset(&mut self) {
        self.0.store(0, Release);
    }
}
impl From<SID> for Seq {
    fn from(value: SID) -> Self {
        Seq(SIDGEN::new(value))
    }
}
/// Represents a timestamped sequence generator.
#[derive(Default)]
pub struct TimestampSeq {
    sequence: AtomicU64,
    last_cycle_timestamp: AtomicU64,
}
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Clock moved backwards!")
        .as_millis() as u64
}
impl TimestampSeq {
    /// Creates a new `TimestampSeq` instance with default values.
    pub fn new() -> TimestampSeq {
        TimestampSeq::default()
    }

    fn wait_next_millis(&self) {
        let last_timestamp = self.last_cycle_timestamp.load(Acquire);
        while get_timestamp() <= last_timestamp {}
    }

    /// Generates the next unique ID based on the timestamp and sequence number.
    ///
    /// The generated ID is a combination of timestamp and sequence number, ensuring uniqueness.
    /// * Note that it is not guaranteed to be in increasing order。
    /// # Examples
    ///
    /// ```
    /// use idable::TimestampSeq;
    ///
    /// let mut timestamp_seq = TimestampSeq::new();
    ///
    /// // Generate the next unique ID.
    /// let unique_id = timestamp_seq.next_id();
    ///
    /// // Print the generated unique ID.
    /// println!("Generated Unique ID: {}", unique_id);
    /// ```
    pub fn next_id(&mut self) -> u64 {
        let sequence = self.sequence.fetch_add(1, SeqCst) & SEQUENCE_MASK;
        let mut new_timestamp = get_timestamp();
        // If the sequence goes one cycle, check if the timestamp hasn't changed yet
        if sequence == 0 {
            let last_timestamp = self.last_cycle_timestamp.load(Acquire);
            if last_timestamp == new_timestamp {
                self.wait_next_millis();
                new_timestamp = get_timestamp();
            }
            self.last_cycle_timestamp.fetch_max(new_timestamp, Release);
        }
        (new_timestamp - EPOCH) << TIMESTAMP_SHIFT | sequence
    }
}
#[cfg(any(test, feature = "for-test"))]
pub fn into_parts(timestamp: u64) -> (u64, u64) {
    (timestamp >> TIMESTAMP_SHIFT, timestamp & SEQUENCE_MASK)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to wait for the next millisecond
    fn wait_for_next_millis() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    #[test]
    fn test_next_generates_unique_ids() {
        let mut timestamp_seq = TimestampSeq::new();

        // Generate multiple unique IDs and ensure they are different.
        let id1 = timestamp_seq.next_id();
        let id2 = timestamp_seq.next_id();
        let id3 = timestamp_seq.next_id();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_next_increases_sequence() {
        let mut timestamp_seq = TimestampSeq::new();

        // Generate IDs and ensure the sequence increases.
        let id1 = timestamp_seq.next_id();
        let id2 = timestamp_seq.next_id();

        assert!(id2 > id1);
    }

    #[test]
    fn test_next_does_not_repeat_ids() {
        let mut timestamp_seq = TimestampSeq::new();

        // Generate multiple IDs and ensure no repetition.
        let id1 = timestamp_seq.next_id();
        let id2 = timestamp_seq.next_id();
        let id3 = timestamp_seq.next_id();
        let id4 = timestamp_seq.next_id();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id3, id4);
        assert_ne!(id1, id4);
        println!("{id1} {id2} {id3} {id4}");
        println!(
            "{:?} {:?} {:?} {:?}",
            into_parts(id1),
            into_parts(id2),
            into_parts(id3),
            into_parts(id4)
        );
    }

    #[test]
    fn test_next_wait_for_next_millis() {
        let mut timestamp_seq = TimestampSeq::new();

        // Generate two IDs in quick succession and ensure the second one has a greater timestamp.
        let id1 = timestamp_seq.next_id();
        wait_for_next_millis();
        let id2 = timestamp_seq.next_id();

        let timestamp1 = id1 >> TIMESTAMP_SHIFT;
        let timestamp2 = id2 >> TIMESTAMP_SHIFT;

        assert!(timestamp2 > timestamp1);
    }
}
