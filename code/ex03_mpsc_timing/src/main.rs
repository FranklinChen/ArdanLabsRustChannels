use std::io::Write;
use std::time::Instant;

enum Message {
    TimeMe(Instant)
}

fn main() {
    // Create a channel
    let (tx, rx) = std::sync::mpsc::sync_channel(65536);

    // Send lots of data
    let mut handles = Vec::new();
    for t in 0 .. 64 {
        // Per-thread sender
        let mut timings = Vec::with_capacity(1024);
        let my_tx = tx.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0 .. 1024 {
                let now = Instant::now();
                my_tx.send(Message::TimeMe(now.clone())).unwrap();
                let elapsed = now.elapsed();
                timings.push(elapsed);
            }

            // Open `/tmp/send_timings.csv` for append
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("/tmp/send_timings.csv")
                .unwrap();
            for (i, timing) in timings.iter().enumerate() {
                let line = format!("{},{},{}\n", t, i, timing.as_nanos());
                file.write_all(line.as_bytes()).unwrap();
            }
        }));
    }

    // Since we've only sent clones, drop the first tx
    drop(tx);

    // Receive until the channel closes
    let mut receive_times = Vec::with_capacity(64 * 1024);
    while let Some(msg) = rx.recv().ok() {
        match msg {
            Message::TimeMe(sent) => receive_times.push(sent.elapsed()),
        }
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Open /tmp/receive_timings.csv for append
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/receive_timings.csv")
        .unwrap();
    for (i, timing) in receive_times.iter().enumerate() {
        let line = format!("{},{}\n", i, timing.as_nanos());
        file.write_all(line.as_bytes()).unwrap();
    }
}
