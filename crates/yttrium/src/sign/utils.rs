// use relay_rpc::domain::MessageId;

// /// Generates unique message IDs for use in RPC requests. Uses 56 bits for the
// /// timestamp with millisecond precision, with the last 8 bits from a monotonic
// /// counter. Capable of producing up to `256000` unique values per second.
// #[derive(Debug, Clone)]
// pub struct MessageIdGenerator {
//     next: Arc<AtomicU8>,
// }

// impl MessageIdGenerator {
//     pub fn new() -> Self {
//         Self { next: Arc::new(AtomicU8::new(0)) }
//     }

//     /// Generates a [`MessageId`].
//     pub fn next(&self) -> MessageId {
//         let next = self.next.fetch_add(1, Ordering::Relaxed) as u64;
//         let timestamp = chrono::Utc::now().timestamp_millis() as u64;
//         let id = (timestamp << 8) | next;

//         MessageId::new(id)
//     }
// }
