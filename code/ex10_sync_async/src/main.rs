use tokio::runtime::Runtime;

async fn async_land(mut rx: tokio::sync::mpsc::Receiver<i32>) {
    tokio::spawn(async move {
        while let Some(v) = rx.recv().await {
            println!("Received: {}", v);
        }
    });
}

fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    // Spawn 10 SYNCHRONOUS threads
    for _ in 0..10 {
        let tx = tx.clone();
        std::thread::spawn(move|| {
            for i in 0..10 {
                tx.blocking_send(i).unwrap();
            }
        });
    }
    drop(tx);

    // Launch into tokio
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async_land(rx));
}
