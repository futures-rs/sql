use std::{
    sync::{Arc, Mutex},
    task::Poll,
};

pub struct Waker<Output> {
    pub waker: Option<std::task::Waker>,
    pub output: Option<Output>,
}

impl<Output> Waker<Output> {
    pub fn poll(&mut self, waker: std::task::Waker) -> Poll<Output> {
        if let Some(output) = self.output.take() {
            return Poll::Ready(output);
        }

        self.waker = Some(waker);

        Poll::Pending
    }

    pub fn ready(&mut self, output: Output) {
        assert!(self.output.is_none(), "call ready function twice");
        self.output = Some(output);

        if let Some(waker) = self.waker.take() {
            waker.wake_by_ref();
        }
    }
}

/// Waker shared between threads
pub type SharedWaker<Output> = Arc<Mutex<Waker<Output>>>;

/// Create new [`SharedWaker`] object
pub fn new_shared_waker<Output>() -> SharedWaker<Output> {
    Arc::new(Mutex::new(Waker::<Output> {
        waker: Default::default(),
        output: None,
    }))
}

/// Future for [`open`](Driver::open) method
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WakableFuture<Output> {
    pub waker: SharedWaker<Output>,
}

impl<Output> WakableFuture<Output> {
    /// Create default new connector object
    pub fn new() -> (Self, SharedWaker<Output>) {
        let waker = new_shared_waker();

        return (
            Self {
                waker: waker.clone(),
            },
            waker,
        );
    }
}

/// Impl Future trait for Connector
impl<Output> std::future::Future for WakableFuture<Output> {
    type Output = Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.waker.lock().unwrap().poll(cx.waker().clone())
    }
}
