use std::sync::atomic::{AtomicU32, AtomicUsize};
use std::sync::Mutex;
use arr_macro::arr;
use crate::gui::MyApp;

mod producer;
mod reporter;
mod processor_level1;
mod processor_level2;
mod gui;

const PRODUCER_CHANNEL_SIZE: usize = 500_000;
const BATCH_CHANNEL_SIZE: usize = 500_000;
const NUM_PRODUCERS: usize = 5;
const NUM_LEVEL1_PROCESSORS: usize = 4;
const NUM_LEVEL2_PROCESSORS: usize = 1;
static BATCH_SIZE: AtomicUsize = AtomicUsize::new(32);
static PROCESSING_DELAY_10TH_SECONDS: AtomicU32 = AtomicU32::new(0);

static PRODUCER_PERFORMANCE: [AtomicU32; 128] = arr![AtomicU32::new(0); 128];
static LAYER1_PERFORMANCE: [AtomicU32; 128] = arr![AtomicU32::new(0); 128];
static LAYER2_PERFORMANCE: [AtomicU32; 128] = arr![AtomicU32::new(0); 128];
static PRODUCER_PERCENT: AtomicU32 = AtomicU32::new(0);
static LAYER1_PERCENT: AtomicU32 = AtomicU32::new(0);

fn main() {
    let (producer_tx, producer_rx) = flume::bounded(PRODUCER_CHANNEL_SIZE);
    let (report_tx, report_rx) = flume::unbounded();
    let (level1_tx, level1_rx) = flume::bounded(BATCH_CHANNEL_SIZE);

    // Spawn reporter
    std::thread::spawn(move || reporter::reporter_thread(report_rx));

    // Spawn producers
    let mut graph_lock = gui::PRODUCER_HISTORY.lock().unwrap();
    for i in 0..NUM_PRODUCERS {
        graph_lock.push(Vec::new());
        let tx = producer_tx.clone();
        let report = report_tx.clone();
        std::thread::spawn(move || producer::producer_thread(i, tx, report));
    }
    drop(graph_lock);

    // Spawn level 1 processors
    let mut graph_lock = gui::LAYER1_HISTORY.lock().unwrap();
    for i in 0..NUM_LEVEL1_PROCESSORS {
        graph_lock.push(Vec::new());
        let input = producer_rx.clone();
        let output = level1_tx.clone();
        let report = report_tx.clone();
        std::thread::spawn(move || processor_level1::processor_1(i, input, output, report));
    }
    drop(graph_lock);

    // Spawn level 2 processors
    let mut graph_lock = gui::LAYER2_HISTORY.lock().unwrap();
    for _i in 0..NUM_LEVEL2_PROCESSORS {
        graph_lock.push(Vec::new());
        let input = level1_rx.clone();
        let report = report_tx.clone();
        std::thread::spawn(move || processor_level2::processor_layer2(input, report));
    }
    drop(graph_lock);

    // Capacity Monitor
    std::thread::spawn(move || {
        loop {
            let producer_len = producer_tx.len();
            let level1_len = level1_tx.len();
            let producer_cap = producer_tx.capacity().unwrap_or(1);
            let level1_cap = level1_tx.capacity().unwrap_or(1);
            let producer_percent = ((producer_len as f32 / producer_cap as f32) * 100.0) as u32;
            let level1_percent = ((level1_len as f32 / level1_cap as f32) * 100.0) as u32;
            PRODUCER_PERCENT.store(producer_percent, std::sync::atomic::Ordering::Relaxed);
            LAYER1_PERCENT.store(level1_percent, std::sync::atomic::Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_secs_f32(0.25));
        }
    });

    // Visualization
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Channel Data-Flow Visualizer",
        options,
        Box::new(|cc| {
            Ok(Box::<MyApp>::default())
        }),
    );
}
