use std::thread;

use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;

lazy_static! {
    pub static ref GLOBAL_HID_EVENT_BUS: OnceCell<HIDEventBus> = OnceCell::new();
}

#[macro_export]
macro_rules! send_hid_event {
    ($event: expr) => {
        GLOBAL_HID_EVENT_BUS.get().unwrap().publish($event)
    };
}

pub fn init() {
    GLOBAL_HID_EVENT_BUS.get_or_init(|| HIDEventBus::new());
}

#[derive(Clone, Copy, Debug)]
pub enum HIDEvent {
    MoveForward(f32),
    MoveBackward(f32),
}

pub struct HIDEventBus {
    sender: Sender<HIDEvent>,
    receiver: Receiver<HIDEvent>,
}

impl HIDEventBus {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn subscribe<F>(&self, callback: &mut F)
    where
        F: FnMut(HIDEvent) + Send + 'static,
    {
        let receiver = self.receiver.clone();
        thread::spawn(move || {
            receiver.into_iter().for_each(|event| {
                callback(event);
            });
        });
    }

    pub fn publish(&self, event: HIDEvent) {
        self.sender.send(event).unwrap()
    }
}
