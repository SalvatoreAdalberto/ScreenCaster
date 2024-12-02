use std::sync::mpsc::{Receiver, Sender};
use crossbeam_channel::{bounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};
use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use ffmpeg_sidecar::event::OutputVideoFrame;
use iced::widget::image::Handle;
use image::{RgbImage, RgbaImage, DynamicImage};


struct Worker{
}

impl Worker{
    fn new(id: usize, rec_frame: CrossbeamReceiver<OutputVideoFrame>, send_image: CrossbeamSender<Handle>, sending_turn_condvar: (Arc<Mutex<usize>>, Arc<Condvar>)) -> Self{
        let (sending_turn, condvar) = sending_turn_condvar;
        let mut handle = Handle::from_pixels(0, 0, Vec::new());
        thread::spawn(move || {
            while let Ok(frame) = rec_frame.recv(){
                let rgb_image: RgbImage = RgbImage::from_raw(frame.width, frame.height, frame.data).expect("Failed to create RgbImage");
                let dynimage = DynamicImage::ImageRgb8(rgb_image);
                // Convert RgbImage to RgbaImage (adding an alpha channel with 255 for full opacity)
                let rgba_image: RgbaImage = dynimage.to_rgba8();
                handle = Handle::from_pixels(frame.width as u32, frame.height as u32, rgba_image.to_vec());
                let mut sending_turn_guard = sending_turn.lock().unwrap();
                while *sending_turn_guard != id {
                    sending_turn_guard = condvar.wait(sending_turn_guard).unwrap();
                }
                send_image.send(handle).unwrap();
            }
        });
        Self{
        }
    }
}

pub struct WorkersManger{
    n_workers: usize,
    cv: Arc<Condvar>,
    sending_turn: Arc<Mutex<usize>>,
    receiving_turn: Mutex<usize>,
    internal_txs_frame: Vec<CrossbeamSender<OutputVideoFrame>>,
    internal_rx_image: CrossbeamReceiver<Handle>,
    receive_frame: Arc<Mutex<Receiver<OutputVideoFrame>>>,
    send_image: Sender<Handle>,
    workers: Vec<Worker>,
}

impl WorkersManger{
    pub fn new(n_workers: usize, receive_frame: Arc<Mutex<Receiver<OutputVideoFrame>>>, send_image: Sender<Handle>) -> Self{
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
            receive_frame,
            send_image,
            workers
        }
    }

    pub fn execute(&self){
        let receiver_frame = self.receive_frame.lock().unwrap();
        while let Ok(frame) = receiver_frame.recv(){
            let mut receiving_turn_guard = self.receiving_turn.lock().unwrap();
            let i = *receiving_turn_guard;
            *receiving_turn_guard = (*receiving_turn_guard + 1) % self.n_workers;
            drop(receiving_turn_guard);
            self.internal_txs_frame[i].send(frame).unwrap();
        }
    }

    pub fn activate(&self){
        while let Ok(processed_image) = self.internal_rx_image.recv() {
            match self.send_image.send(processed_image){
                Ok(_) => {
                    let mut sending_turn_guard = self.sending_turn.lock().unwrap();
                    *sending_turn_guard = (*sending_turn_guard + 1) % self.n_workers;
                    drop(sending_turn_guard);
                    self.cv.notify_all();
                }
                Err(e) => {
                    eprintln!("Error sending processed image: {}", e);
                    break;
                }
            }

            
        }
    }

    // fn join(self){
    //     for worker in self.workers{
    //         worker.handle.join().unwrap();
    //     }
    // }
}