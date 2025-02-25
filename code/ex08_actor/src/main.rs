enum ActorMessage {
    RecordData(u64),
    GetData(tokio::sync::oneshot::Sender<u64>),
}

async fn actor(mut rx: flume::Receiver<ActorMessage>) {
    let mut data = 0u64;
    loop {
        match rx.recv_async().await {
            Ok(ActorMessage::RecordData(d)) => {
                data += d;
            }
            Ok(ActorMessage::GetData(sender)) => {
                let _ = sender.send(data);
            }
            Err(_) => {
                break;
            }
        }
    }
}

async fn query_data(tx: flume::Sender<ActorMessage>) -> u64 {
    let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
    tx.send_async(ActorMessage::GetData(oneshot_tx)).await.unwrap();
    oneshot_rx.await.unwrap()
}

#[tokio::main]
async fn main() {
    // Create a channel
    let (tx, rx) = flume::unbounded();

    // Spawn our actor
    tokio::spawn(actor(rx.clone()));

    // Spawn a process that keeps adding data
    let my_tx = tx.clone();
    tokio::spawn(async move {
        for i in 0..1024 {
            my_tx.send_async(ActorMessage::RecordData(i)).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.1)).await;
        }
    });

    // Let's get the data a few times
    for _ in 0..10 {
        let data = query_data(tx.clone()).await;
        println!("Data: {}", data);
        // Sleep
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.2)).await;
    }
}
