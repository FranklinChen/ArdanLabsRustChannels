use std::io::Write;
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, Sender};

enum Message {
    TimeMe(Instant)
}

async fn per_task_sender(t: i32, my_tx: Sender<Message>) {
    let mut timings = Vec::with_capacity(1024);
    for _ in 0 .. 1024*64 {
        let now = Instant::now();
        my_tx.send(Message::TimeMe(now.clone())).await.unwrap();
        let elapsed = now.elapsed();
        timings.push(elapsed);
    }

    // Open `/tmp/send_timings_async.csv` for append
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/send_timings_async_u.csv")
        .unwrap();
    for (i, timing) in timings.iter().enumerate() {
        let line = format!("{},{},{}\n", t, i, timing.as_nanos());
        file.write_all(line.as_bytes()).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(65536);

    // Send lots of data
    let my_tx = tx.clone();
    tokio::spawn(per_task_sender(1, my_tx));

    // Since we've only sent clones, drop the first tx
    drop(tx);

    // Receive until the channel closes
    let mut receive_times = Vec::with_capacity(64 * 1024);
    while let Some(msg) = rx.recv().await {
        match msg {
            Message::TimeMe(sent) => receive_times.push(sent.elapsed()),
        }
    }

    // Open /tmp/receive_timings.csv for append
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/receive_timings_async_u.csv")
        .unwrap();
    for (i, timing) in receive_times.iter().enumerate() {
        let line = format!("{},{}\n", i, timing.as_nanos());
        file.write_all(line.as_bytes()).unwrap();
    }
}
