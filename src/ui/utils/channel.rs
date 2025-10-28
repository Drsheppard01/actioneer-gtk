use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use gtk4::glib::{ControlFlow, MainContext, Priority};

pub struct Sender<T: Send + 'static>(UnboundedSender<T>);

pub struct Receiver<T: Send + 'static>(UnboundedReceiver<T>);

pub trait MainContextChannelExt {
    fn channel<T: Send + 'static>(&self, priority: Priority) -> (Sender<T>, Receiver<T>);
}

impl MainContextChannelExt for MainContext {
    fn channel<T: Send + 'static>(&self, _priority: Priority) -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = mpsc::unbounded();
        (Sender(tx), Receiver(rx))
    }
}

impl<T: Send + 'static> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), mpsc::TrySendError<T>> {
        self.0.unbounded_send(value)
    }
}

impl<T: Send + 'static> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Send + 'static> Receiver<T> {
    pub fn attach<F>(self, _context: Option<&MainContext>, mut func: F)
    where
        F: FnMut(T) -> ControlFlow + 'static,
    {
        let mut receiver = self.0;
        MainContext::default().spawn_local(async move {
            loop {
                let next_item = receiver.next().await;
                let Some(msg) = next_item else {
                    break;
                };

                if matches!(func(msg), ControlFlow::Break) {
                    break;
                }
            }
        });
    }
}
