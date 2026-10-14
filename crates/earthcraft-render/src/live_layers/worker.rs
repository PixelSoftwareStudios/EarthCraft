use super::{LiveGeometry, LiveInput, LivePreparer};
use std::sync::{Arc, Condvar, Mutex};
#[derive(Default)]
struct Mailbox {
    pending: Option<LiveInput>,
    completed: Option<Arc<LiveGeometry>>,
    stopped: bool,
}
#[derive(Default)]
struct Shared {
    mailbox: Mutex<Mailbox>,
    wake: Condvar,
}
pub struct LiveWorker {
    shared: Arc<Shared>,
}
impl LiveWorker {
    pub fn new(repaint: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        let shared = Arc::new(Shared::default());
        let worker = shared.clone();
        std::thread::Builder::new()
            .name("earthcraft-live-layers".into())
            .spawn(move || {
                let mut preparer = LivePreparer::default();
                loop {
                    let input = {
                        let mut state = worker.mailbox.lock().unwrap();
                        while state.pending.is_none() && !state.stopped {
                            state = worker.wake.wait(state).unwrap();
                        }
                        if state.stopped {
                            return;
                        }
                        state.pending.take().unwrap()
                    };
                    let geometry = preparer.prepare(input);
                    let mut state = worker.mailbox.lock().unwrap();
                    if state.stopped {
                        return;
                    }
                    let previous = state.completed.replace(geometry);
                    drop(state);
                    drop(previous);
                    repaint();
                }
            })?;
        Ok(Self { shared })
    }
    pub fn submit(&self, input: LiveInput) {
        let previous = self.shared.mailbox.lock().unwrap().pending.replace(input);
        drop(previous);
        self.shared.wake.notify_one();
    }
    pub fn take_completed(&self) -> Option<Arc<LiveGeometry>> {
        self.shared.mailbox.lock().unwrap().completed.take()
    }
}
impl Drop for LiveWorker {
    fn drop(&mut self) {
        let mut state = self.shared.mailbox.lock().unwrap();
        state.stopped = true;
        let pending = state.pending.take();
        let completed = state.completed.take();
        drop(state);
        self.shared.wake.notify_one();
        drop((pending, completed));
    }
}
