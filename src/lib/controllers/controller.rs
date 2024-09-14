// use std::{sync::{mpsc::Receiver, Arc}, thread};

// pub trait Controller<T> {
//     fn start(&self, rx: Arc<Mutex<Receiver<T>>>) {
//         thread::spawn(move || {
//             Self::function(rx.into());
//         });
//     }

//     fn stop(&self) {

//     }

//     fn function(rx: Arc<Receiver<T>>) {}
// }