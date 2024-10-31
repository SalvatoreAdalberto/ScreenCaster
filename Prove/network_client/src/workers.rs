use std::sync::mpsc::{self, Receiver, Sender};
use crossbeam_channel::{bounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};
use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use ffmpeg_sidecar::event::OutputVideoFrame;
use eframe::egui::ColorImage;

struct Worker{
    handle: thread::JoinHandle<()>,
}

impl Worker{
    fn new(id: usize, rec_frame: CrossbeamReceiver<OutputVideoFrame>, send_image: CrossbeamSender<ColorImage>, sending_turn_condvar: (Arc<Mutex<usize>>, Arc<Condvar>)) -> Self{
        let (sending_turn, condvar) = sending_turn_condvar;
        let handle = thread::spawn(move || {
            while let Ok(frame) = rec_frame.recv(){
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
    internal_txs_frame: Vec<CrossbeamSender<OutputVideoFrame>>,
    internal_rx_image: CrossbeamReceiver<ColorImage>,
    send_image: Sender<ColorImage>,
    workers: Vec<Worker>,
}

impl WorkersManger{
    pub fn new(n_workers: usize, send_image: Sender<ColorImage>) -> Self{
        let cv = Arc::new(Condvar::new());
        let sending_turn = Arc::new(Mutex::new(0));
        let receiving_turn = Mutex::new(0);
        let mut workers = Vec::new();
        let mut internal_txs_frame = Vec::new();
        let (internal_tx_image, internal_rx_image) = bounded(5);
        
        for i in 0..n_workers{
            let (internal_tx_frame, internal_rx_frame) = bounded(1);
            internal_txs_frame.push(internal_tx_frame);
            let worker = Worker::new(
                    i, 
                    internal_rx_frame,
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
            internal_rx_image,
            internal_txs_frame,
            send_image,
            workers
        }
    }

    pub fn execute(&self, frame: OutputVideoFrame){
        let mut receiving_turn_guard = self.receiving_turn.lock().unwrap();
        let i = *receiving_turn_guard;
        *receiving_turn_guard = (*receiving_turn_guard + 1) % self.n_workers;
        drop(receiving_turn_guard);
        self.internal_txs_frame[i].send(frame).unwrap();
    }

    pub fn activate(&self){
        while let Ok(processed_image) = self.internal_rx_image.recv() {
            self.send_image.send(processed_image).unwrap();
            let mut sending_turn_guard = self.sending_turn.lock().unwrap();
            *sending_turn_guard = (*sending_turn_guard + 1) % self.n_workers;
            drop(sending_turn_guard);
            self.cv.notify_all();
        }
    }

    fn join(self){
        for worker in self.workers{
            worker.handle.join().unwrap();
        }
    }
}


// ATTEMPT OF IMPLEMENTING A BUFFER IN THE WORKER MANAGER (ACTIVATE METHOD)
// let mut frame_counter = 0;
        // let buffer_size = 240;
        // let receiving_buffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
        // let sending_buffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
//     {
        //         let mut receiving_buffer_guard = receiving_buffer.lock().unwrap();
        //         receiving_buffer_guard.push_back(processed_image);
        //         frame_counter += 1;
        //     }
        //     if frame_counter >= buffer_size {
        //         // Move frames from receiving buffer to sending buffer
        //         {
        //             let mut receiving_buffer_guard = receiving_buffer.lock().unwrap();
        //             let mut sending_buffer_guard = sending_buffer.lock().unwrap();
        //             while let Some(frame) = receiving_buffer_guard.pop_front() {
        //                 sending_buffer_guard.push_back(frame);
        //             }
        //         }
        //         frame_counter = 0;
        //         // Send frames from the sending buffer in a separate thread
        //         let send_image = self.send_image.clone();
        //         let sending_buffer_clone = Arc::clone(&sending_buffer);
        //         thread::spawn(move || {
        //             let mut sending_buffer_guard = sending_buffer_clone.lock().unwrap();
        //             while let Some(frame) = sending_buffer_guard.pop_front() {
        //                 send_image.send(frame).unwrap();
        //             }
        //         });
        //     }