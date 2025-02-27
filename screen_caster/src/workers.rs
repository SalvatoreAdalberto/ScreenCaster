use std::sync::mpsc::{Receiver, Sender};
use crossbeam_channel::{bounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex, Condvar};
use ffmpeg_sidecar::event::OutputVideoFrame;
use iced::widget::image::Handle;
use image::{RgbImage, RgbaImage, DynamicImage};


/* 
This module is used to process raw format frames coming from ffmpeg process and convert them into 
Iced Handle, in order to be displayed. The FrameProcessor concurrently processes frames using multiple workers.
Each worker process a different frame and waits its turn to deliver the processed image, in such a way to mantain the
relative order of the frames.

EXAMPLE OF THE FRAME PROCESSOR ARCHITECTURE, 3 WORKERS

                              |
                              V
+---------------------------------------------------------------+
|                         Dispatcher (D)                        |
|                                                               |
|                                                               |
+---------------------------------------------------------------+
        |                     |                     |
        v                     v                     v
+-----------------+   +-----------------+   +-----------------+
|   Channel       |   |   Channel       |   |   Channel       |
|    W1           |   |    W2           |   |    W3           |
|                 |   |                 |   |                 |
+-----------------+   +-----------------+   +-----------------+
        |                     |                     |
        v                     v                     v
  +------------+         +------------+        +------------+
  |  Worker 1  |         |  Worker 2  |        |  Worker 3  |
  |     W1     |         |     W2     |        |     W3     |
  +------------+         +------------+        +------------+
        |                     |                     |
        v                     v                     v
    +------------------------------------------------------+         
    |                    Shared Channel                    |         
    |                                                      |         
    +------------------------------------------------------+         
                             |
                             v
                    +-------------------+
                    |   Aggregator (A)  |
                    +-------------------+
                             |
                             v
                        
*/



/// The FrameDispatcher is responsible for receiving frames from the ffmpeg and 
/// dispatching them to the FrameProcessorWorkers.

pub struct FrameDispatcher{
    n_workers: usize,
    receiver_frame: Receiver<OutputVideoFrame>,
    terminate_receiver: CrossbeamReceiver<bool>,
    internal_txs_frame: Vec<CrossbeamSender<OutputVideoFrame>>,
}

impl FrameDispatcher{
    fn new(n_workers: usize, receiver_frame: Receiver<OutputVideoFrame>, terminate_receiver: CrossbeamReceiver<bool>, 
        internal_txs_frame: Vec<CrossbeamSender<OutputVideoFrame>>) -> Self{
        Self{
            n_workers,
            receiver_frame,
            terminate_receiver,
            internal_txs_frame
        }
    }
    pub fn execute(self){
        let mut receiving_turn = 0;
        while let Ok(frame) = self.receiver_frame.recv(){
            let i = receiving_turn;
            receiving_turn = (receiving_turn + 1) % self.n_workers;
            if let Ok(true) = self.terminate_receiver.try_recv(){
                break;
            }
            match self.internal_txs_frame[i].send(frame){
                Ok(_) => {},
                Err(_) => {break}
            }
        }
    }
    
}


///The FrameAggregator is responsible for receiving processed images from the FrameProcessorWorkers 
/// and sending them to the iced GUI. 
/// It notifies the FrameProcessorWorkers when it is their turn to send the processed image, in order to receive them in order to be played.
pub struct FrameAggregator{
    internal_rx_image: Option<CrossbeamReceiver<Handle>>,
    send_image: Sender<Handle>,
    sending_turn:  Arc<Mutex<usize>>,
    n_workers: usize,
    cv: Arc<Condvar>,
    terminate_sender: CrossbeamSender<bool>,
    workers: Vec<FrameProcessorWorker>,
}

impl FrameAggregator{
    fn new( n_workers: usize, internal_rx_image: CrossbeamReceiver<Handle>, send_image: Sender<Handle>, sending_turn:  Arc<Mutex<usize>>,
            cv: Arc<Condvar>, terminate_sender: CrossbeamSender<bool>, workers: Vec<FrameProcessorWorker>) -> Self{
                Self{
                    internal_rx_image: Some(internal_rx_image),
                    send_image,
                    sending_turn,
                    n_workers,
                    cv,
                    terminate_sender,
                    workers
                }
    }

    pub fn activate(&mut self){
        let internal_rx_image = self.internal_rx_image.clone().unwrap();
        self.internal_rx_image = None;
        while let Ok(processed_image) = internal_rx_image.recv() {
            match self.send_image.send(processed_image){
                Ok(_) => {
                    let mut sending_turn_guard = self.sending_turn.lock().unwrap();
                    *sending_turn_guard = (*sending_turn_guard + 1) % self.n_workers;
                    drop(sending_turn_guard);
                    self.cv.notify_all();
                }
                Err(_) => {
                    drop(internal_rx_image);
                    self.terminate_sender.send(true).unwrap();
                    while let Ok(_) = self.terminate_sender.send(true){
                        let mut sending_turn_guard = self.sending_turn.lock().unwrap();
                        *sending_turn_guard = (*sending_turn_guard + 1) % self.n_workers;
                        drop(sending_turn_guard);
                        self.cv.notify_all();
                    }
                    break;
                }
            } 
        }
    }

    pub fn join_workers(self){
        for worker in self.workers{
            worker.thread_handle.join().unwrap();
        }
    }
}

///The FrameProcessorWorkers are responsible for processing the frames and sending the processed
/// images to the FrameAggregator.
struct FrameProcessorWorker{
    thread_handle: JoinHandle<()>,
}

impl FrameProcessorWorker{
    fn new(id: usize, rec_frame: CrossbeamReceiver<OutputVideoFrame>, send_image: CrossbeamSender<Handle>, sending_turn_condvar: (Arc<Mutex<usize>>, Arc<Condvar>), terminate_receiver: CrossbeamReceiver<bool>) -> Self{
        let (sending_turn, condvar) = sending_turn_condvar;
        let mut handle = Handle::from_pixels(0, 0, Vec::new());
        let thread_handle = thread::spawn(move || {
            'main: while let Ok(frame) = rec_frame.recv(){
                    let rgb_image: RgbImage = RgbImage::from_raw(frame.width, frame.height, frame.data).expect("Failed to create RgbImage");
                    let dynimage = DynamicImage::ImageRgb8(rgb_image);
                    // Convert RgbImage to RgbaImage (adding an alpha channel with 255 for full opacity)
                    let rgba_image: RgbaImage = dynimage.to_rgba8();
                    handle = Handle::from_pixels(frame.width as u32, frame.height as u32, rgba_image.to_vec());
                    
                    if let Ok(true) = terminate_receiver.try_recv(){
                        break;
                    }
                    let mut sending_turn_guard: std::sync::MutexGuard<'_, usize> = sending_turn.lock().unwrap();
                    while *sending_turn_guard != id {
                        sending_turn_guard = condvar.wait(sending_turn_guard).unwrap();
                        if let Ok(true) = terminate_receiver.try_recv(){
                            break 'main ;
                        }
                    }
                    match send_image.send(handle){
                        Ok(_) => {},
                        Err(_) => {
                            break;
                        },
                    }
                }
        });
        Self{
            thread_handle,
        }
    }
}

///The FrameProcessorConstructor is a factory that creates all necessary structs for FrameProcessor to work:
/// FramePRoccessorWorkers, FrameDispatcher and a FrameAggregator.
pub struct FrameProcessorConstructor{
}

impl FrameProcessorConstructor{
    pub fn new(n_workers: usize, receiver_frame: Receiver<OutputVideoFrame>, send_image: Sender<Handle>) -> (FrameDispatcher, FrameAggregator){
        let cv = Arc::new(Condvar::new());
        let sending_turn = Arc::new(Mutex::new(0));
        let mut workers = Vec::new();
        let mut internal_txs_frame = Vec::new();

        let (internal_tx_image, internal_rx_image) = bounded(5);
        let (terminate_sender, terminate_receiver) = bounded(1);
        
        for i in 0..n_workers{
            let (internal_tx_frame, internal_rx_frame) = bounded(5);
            internal_txs_frame.push(internal_tx_frame);
            let worker = FrameProcessorWorker::new(
                    i, 
                    internal_rx_frame,
                    internal_tx_image.clone(),
                    (sending_turn.clone(), cv.clone()),
                    terminate_receiver.clone(),
                );
            workers.push(worker);
        }

        (FrameDispatcher::new(n_workers, receiver_frame, terminate_receiver, internal_txs_frame),
        FrameAggregator::new(n_workers, internal_rx_image, send_image, sending_turn, cv,terminate_sender, workers))
            
    }


    
}
