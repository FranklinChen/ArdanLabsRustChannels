use crate::PROCESSING_DELAY_10TH_SECONDS;
use std::time::Duration;

pub fn processor_layer2(
    from_layer1: flume::Receiver<Vec<u64>>,
    report: flume::Sender<crate::reporter::Report>,
) {
    let mut count = 0;
    let mut start = std::time::Instant::now();
    while let Ok(_batch) = from_layer1.recv() {
        // Simulate processing time
        let processing_delay =
            PROCESSING_DELAY_10TH_SECONDS.load(std::sync::atomic::Ordering::Relaxed) as f32 / 10.0;
        std::thread::sleep(Duration::from_secs_f32(processing_delay));

        count += 1;
        let elapsed_seconds = start.elapsed().as_secs_f32();
        if elapsed_seconds >= 0.25 {
            let messages_per_second = count as f32 / elapsed_seconds;
            let _ = report.send(crate::reporter::Report::Layer2(0, messages_per_second));
            count = 0;
            start = std::time::Instant::now();
        }
    }
    println!("Layer 2 processor exiting");
}
