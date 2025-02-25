fn main() {
    // Create a channel
    let (tx, rx) = std::sync::mpsc::channel();

    // Create a thread and wait for messages
    let thread = std::thread::spawn(move || {
        let message = rx.recv().unwrap();
        print!("Received: {}", message);
    });

    // Submit a message
    tx.send("Hello, World!").unwrap();

    // Wait for the thread to finish
    thread.join().unwrap();
}
