```rs
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
```