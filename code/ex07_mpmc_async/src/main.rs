use flume::{Receiver, Sender};
use std::io::Write;
use std::time::Instant;
use tokio::join;

enum Message {
    TimeIt(Instant),
}

async fn receiver(rx: Receiver<Message>) -> Vec<u128> {
    let mut recv_times = Vec::with_capacity(65536);
    while let Ok(msg) = rx.recv_async().await {
        match msg {
            Message::TimeIt(start) => {
                let elapsed = start.elapsed();
                recv_times.push(elapsed);
            }
        }
    }
    recv_times.into_iter().map(|t| t.as_nanos()).collect()
}

async fn sender(tx: Sender<Message>) -> Vec<u128> {
    let mut send_times = Vec::with_capacity(65536);
    for _ in 0..1024 {
        let start = Instant::now();
        tx.send_async(Message::TimeIt(start)).await.unwrap();
        let elapsed = start.elapsed();
        send_times.push(elapsed);
    }
    send_times.into_iter().map(|t| t.as_nanos()).collect()
}

#[tokio::main]
async fn main() {
    let (tx, rx) = flume::bounded(65536);

    // Spawn 64 senders
    let mut handles = Vec::new();
    for _t in 0..64 {
        let tx = tx.clone();
        let handle = tokio::spawn(async move { sender(tx).await });
        handles.push(handle);
    }
    drop(tx);

    // Join 4 receivers
    let recv_times = join!(
        receiver(rx.clone()),
        receiver(rx.clone()),
        receiver(rx.clone()),
        receiver(rx.clone())
    );
    let mut recv_time_flat: Vec<u128> = vec![];
    recv_time_flat.extend(recv_times.0);
    recv_time_flat.extend(recv_times.1);
    recv_time_flat.extend(recv_times.2);
    recv_time_flat.extend(recv_times.3);

    // Write the receiver times to `/tmp/flume_recv.csv`
    let mut recv_file = std::fs::File::create("/tmp/flume_recv.csv").unwrap();
    for (i, recv_time) in recv_time_flat.iter().enumerate() {
        writeln!(recv_file, "{i}, {}", recv_time).unwrap();
    }

    // Write the sender times to `/tmp/flume_send.csv`
    let mut send_file = std::fs::File::create("/tmp/flume_send.csv").unwrap();
    for h in handles {
        let send_time = h.await.unwrap();
        for (i, send_time) in send_time.iter().enumerate() {
            writeln!(send_file, "{i}, {}", send_time).unwrap();
        }
    }
}
