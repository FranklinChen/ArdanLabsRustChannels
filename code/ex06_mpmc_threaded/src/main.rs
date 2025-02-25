fn main() {
    let (tx, rx) = flume::bounded(65536);

    // Send lots of messages in 64 threads
    let mut handles = Vec::new();
    for t in 0 .. 64 {
        let tx = tx.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0 .. 1024 {
                tx.send((t, i)).unwrap();
            }
        }));
    }
    drop(tx); // Prevent the original sender from sticking around

    // Receive between 4 recipients.
    for t in 0 .. 4 {
        let rx = rx.clone();
        handles.push(std::thread::spawn(move || {
            for (thread, i) in rx.iter() {
                println!("Thread {} received {},{}", t, thread, i);
            }
        }));
    }
    drop(rx); // Prevent the original receiver from sticking around

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}
