use std::time::{Duration, Instant};

use anyhow::Result;

pub struct Sender<T> {
    tx: crossbeam_channel::Sender<(T, crossbeam_channel::Receiver<Instant>)>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<T> Sender<T>
where
    T: Send + Sync + 'static,
{
    pub fn send(&self, msg: T, delay: Duration) -> Result<()> {
        self.tx.send((msg, crossbeam_channel::after(delay)))?;
        Ok(())
    }
}

pub struct Receiver<T> {
    rx: crossbeam_channel::Receiver<(T, crossbeam_channel::Receiver<Instant>)>,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.clone(),
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T> {
        let (mut last_msg, delay) = self.rx.recv()?;
        delay.recv()?;
        while let Ok((msg, delay)) = self.rx.try_recv() {
            delay.recv()?;
            last_msg = msg;
        }

        Ok(last_msg)
    }
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (Sender { tx }, Receiver { rx })
}
