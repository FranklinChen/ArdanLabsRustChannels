use flume::{Receiver, Sender};
use tokio::select;

enum Sensor {
    Reading(f32),
}

enum Command {
    Stop,
}

async fn sensor(tx: Sender<Sensor>) {
    // Keep sending until the channel is closed
    let mut i = 0.0;
    loop {
        i += 1.0;
        let result = tx.send_async(Sensor::Reading(i)).await;
        if result.is_err() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.001)).await;
    }
}

async fn receiver(sensor_rx: Receiver<Sensor>, control_rx: Receiver<Command>) {
    let mut accumulator = 0.0;
    loop {
        select! {
            sensor = sensor_rx.recv_async() => {
                match sensor {
                    Ok(Sensor::Reading(value)) => {
                        accumulator += value;
                    }
                    Err(_) => {
                        println!("Sensor channel closed");
                        break;
                    }
                }
            }
            command = control_rx.recv_async() => {
                match command {
                    Ok(Command::Stop) => {
                        println!("Received stop command");
                        println!("Accumulator: {}", accumulator);
                        break;
                    }
                    Err(_) => {
                        println!("Control channel closed");
                        break;
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Make a sensor channel
    let (sensor_tx, sensor_rx) = flume::bounded(65536);

    // Make a command channel
    let (command_tx, command_rx) = flume::bounded(1);

    // Start the sensor
    for _ in 0..10 {
        tokio::spawn(sensor(sensor_tx.clone()));
    }

    // Receive Loop
    let receiver = tokio::spawn(receiver(sensor_rx, command_rx));

    // Wait for a while
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    command_tx.send_async(Command::Stop).await.unwrap();
    receiver.await.unwrap();
}
