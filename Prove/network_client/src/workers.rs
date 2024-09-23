use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use ffmpeg_sidecar::event::OutputVideoFrame;
use eframe::egui::ColorImage;

struct Worker{
    handle: thread::JoinHandle<()>,
}

impl Worker{
    fn new(id: usize, rec_frame: Arc<Mutex<Receiver<OutputVideoFrame>>>, send_image: Sender<ColorImage>, sending_turn_condvar: (Arc<Mutex<usize>>, Arc<Condvar>)) -> Self{
        let (sending_turn, condvar) = sending_turn_condvar;
        let handle = thread::spawn(move || {
            while let Ok(frame) = rec_frame.lock().unwrap().recv(){
                let image = ColorImage::from_rgb([frame.width as usize, frame.height as usize], &frame.data);
                let mut sending_turn_guard = sending_turn.lock().unwrap();
                while *sending_turn_guard != id {
                    sending_turn_guard = condvar.wait(sending_turn_guard).unwrap();
                }
                send_image.send(image).unwrap();
            }
        });
        Self{
            handle,
        }
    }
}

pub struct WorkersManger{
    n_workers: usize,
    cv: Arc<Condvar>,
    sending_turn: Arc<Mutex<usize>>,
    receiving_turn: Mutex<usize>,
    internal_txs_frame: Vec<Sender<OutputVideoFrame>>,
    internal_rx_image: Arc<Mutex<Receiver<ColorImage>>>,
    send_image: Sender<ColorImage>,
    workers: Vec<Worker>,
}

impl WorkersManger{
    pub fn new(n_workers: usize, send_image: Sender<ColorImage>) -> Self{
        let cv = Arc::new(Condvar::new());
        let sending_turn = Arc::new(Mutex::new(0));
        let receiving_turn: Mutex<usize> = Mutex::new(0);
        let mut workers = Vec::new();
        let mut internal_txs_frame = Vec::new();
        let (internal_tx_image, internal_rx_image) = mpsc::channel();
        
        for i in 0..n_workers{
            let (internal_tx_frame, internal_rx_frame) = mpsc::channel();
            internal_txs_frame.push(internal_tx_frame);
            let worker = Worker::new(
                    i, 
                    Arc::new(Mutex::new(internal_rx_frame)),
                    internal_tx_image.clone(),
                    (sending_turn.clone(), cv.clone())
                );
            workers.push(worker);
        }

        Self{
            n_workers,
            cv,
            sending_turn,
            receiving_turn,
            internal_rx_image: Arc::new(Mutex::new(internal_rx_image)),
            internal_txs_frame,
            send_image,
            workers
        }
    }

    pub fn execute(&self, frame: OutputVideoFrame){
        let mut receiving_turn_guard = self.receiving_turn.lock().unwrap();
        let i = *receiving_turn_guard;
        *receiving_turn_guard = (*receiving_turn_guard + 1) % self.n_workers;
        self.internal_txs_frame[i].send(frame).unwrap();

    }

    pub fn activate(&self){
        let mut frame_counter = 0;
        let mut frames = Vec::new();

        while let Ok(processed_image) = self.internal_rx_image.lock().unwrap().recv() {
            if frame_counter < 60 {
                frames.push(processed_image);
                frame_counter += 1;
            } else {
                if !frames.is_empty() {
                    for frame in frames.drain(..) {
                        self.send_image.send(frame).unwrap();
                    }
                }
                self.send_image.send(processed_image).unwrap();
            }
        let mut sending_turn_guard = self.sending_turn.lock().unwrap();
        *sending_turn_guard = (*sending_turn_guard + 1) % self.n_workers;
        self.cv.notify_all();
        }
}

    fn join(self){
        for worker in self.workers{
            worker.handle.join().unwrap();
        }
    }
}
