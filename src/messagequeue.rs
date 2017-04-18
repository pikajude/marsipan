use futures::Async;
use futures::Future;
use futures::Stream;
use futures::task;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use std::ops::Deref;
use tokio_core::reactor::Handle;
use tokio_core::reactor::Timeout;

use damnpacket::Message;
use futures;
use MarsError;

#[derive(Debug)]
pub struct Countdown<A> {
    stamp: Instant,
    value: A,
}

impl<A> Countdown<A> {
    fn at(instant: Instant, value: A) -> Self {
        Countdown { stamp: instant, value: value }
    }
}

impl<A> Deref for Countdown<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<A> PartialEq for Countdown<A> {
    fn eq(&self, other: &Self) -> bool {
        self.stamp.eq(&other.stamp)
    }
}

impl<A> Eq for Countdown<A> {}

/// Countdown orders in reverse so the heap becomes a min-heap instead of a max-heap.
impl<A> PartialOrd for Countdown<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A> Ord for Countdown<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.stamp.cmp(&self.stamp)
    }
}

struct MQ {
    heap: BinaryHeap<Countdown<Message>>,
    timeout: Option<Timeout>,
    handle: Handle
}

#[derive(Clone)]
pub struct MessageQueue(Rc<RefCell<MQ>>);

impl MQ {
    fn new(h: &Handle) -> Self {
        MQ {
            heap: BinaryHeap::new(),
            timeout: None,
            handle: h.clone()
        }
    }

    fn push(&mut self, msg: Message) {
        self.schedule_at(msg, Instant::now());
    }

    fn schedule(&mut self, msg: Message, d: Duration) {
        self.schedule_at(msg, Instant::now() + d);
    }

    fn schedule_at(&mut self, msg: Message, ins: Instant) {
        self.heap.push(Countdown::at(ins, msg));
        self.reschedule();
    }

    fn reschedule(&mut self) {
        if let Some(soonest) = self.heap.peek() {
            let i = Instant::now();
            self.timeout = Some(Timeout::new(if soonest.stamp < i {
                Duration::new(0,0)
            } else {
                soonest.stamp - i
            }, &self.handle).unwrap());
            task::park().unpark();
        } else {
            self.timeout = None;
        }
    }

    fn poll(&mut self) -> futures::Poll<Option<Message>, MarsError> {
        let mut removed_item = false;
        let status = match self.timeout {
            None => Ok(Async::NotReady),
            Some(ref mut t) => match t.poll() {
                Ok(Async::Ready(_)) => {
                    removed_item = true;
                    let soonest = self.heap.pop().expect("Invariant: timeout with empty heap").value;
                    Ok(Async::Ready(Some(soonest)))
                },
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => Err(MarsError::from(e))
            }
        };
        if removed_item {
            self.reschedule();
        }
        status
    }
}

impl MessageQueue {
    pub fn push(self, msg: Message) {
        self.0.borrow_mut().push(msg)
    }

    pub fn schedule(self, msg: Message, d: Duration) {
        self.0.borrow_mut().schedule(msg, d)
    }

    pub fn schedule_at(self, msg: Message, ins: Instant) {
        self.0.borrow_mut().schedule_at(msg, ins)
    }

    pub fn new(h: &Handle) -> Self {
        MessageQueue(Rc::new(RefCell::new(MQ::new(h))))
    }
}

impl Stream for MessageQueue {
    type Item = Message;
    type Error = MarsError;

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        self.0.borrow_mut().poll()
    }
}
