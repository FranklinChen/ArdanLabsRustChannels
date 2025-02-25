enum Message {
    Print(String),
}

fn main() {
    // Create a channel
    let (tx, rx) = std::sync::mpsc::sync_channel(65536);

    // Send lots of data
    let mut handles = Vec::new();
    for t in 0 .. 64 {
        // Per-thread sender
        let my_tx = tx.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0 .. 1024 {
                my_tx.send(Message::Print(format!("Thread: {}, Message: {}\n", t, i))).unwrap();
            }
        }));
    }

    // Since we've only sent clones, drop the first tx
    drop(tx);

    // Receive until the channel closes
    while let Some(msg) = rx.recv().ok() {
        match msg {
            Message::Print(s) => print!("{}", s),
        }
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}
