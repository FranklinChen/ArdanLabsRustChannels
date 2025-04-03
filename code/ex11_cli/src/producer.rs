use crate::reporter::Report;

/// Layer 1 data producer. Emits messages as fast as it can, and
/// reports the number of messages sent per second.
pub fn producer_thread(id: usize, tx: flume::Sender<u64>, report: flume::Sender<Report>) {
    let mut start = std::time::Instant::now();
    let mut count = 0;
    loop {
        // Random number
        let result = tx.send(1);
        if result.is_err() {
            break;
        }
        count += 1;
        let elapsed_seconds = start.elapsed().as_secs_f32();
        if elapsed_seconds >= 0.1 {
            let messages_per_second = count as f32 / elapsed_seconds;
            let _ = report.send(Report::Producer(id, messages_per_second));
            count = 0;
            start = std::time::Instant::now();
        }
    }
}
