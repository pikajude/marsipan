use futures::Async;
use futures::Future;
use futures::Stream;
use futures::task;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use tokio_core::reactor::Handle;
use tokio_core::reactor::Timeout;

use damnpacket::Message;
use futures;
use MarsError;

struct MQ {
    heap: FakeHeap<Instant, Message>,
    timeout: Option<Timeout>,
    handle: Handle
}

struct FakeHeap<K,V> {
    _map: BTreeMap<K,V>
}

impl<K,V> FakeHeap<K,V> where K: Ord {
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        self._map.insert(k,v)
    }

    fn peek(&self) -> Option<&K> {
        self._map.keys().next()
    }

    fn pop(&mut self) -> Option<(K, V)> where K: Clone + Copy {
        if let Some(k) = self.peek().cloned() {
            return Some((k, self._map.remove(&k).unwrap()))
        }
        None
    }
}

#[derive(Clone)]
pub struct MessageQueue(Rc<RefCell<MQ>>);

impl MQ {
    fn new(h: &Handle) -> Self {
        MQ {
            heap: FakeHeap {
                _map: BTreeMap::new()
            },
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
        self.heap.insert(ins, msg);
        self.reschedule();
    }

    fn reschedule(&mut self) {
        if let Some(stamp) = self.heap.peek().cloned() {
            let i = Instant::now();
            self.timeout = Some(Timeout::new(if stamp < i {
                Duration::new(0,0)
            } else {
                stamp - i
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
                    let soonest = self.heap.pop().expect("Invariant: timeout with empty heap").1;
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
    pub fn push(&self, msg: Message) {
        self.0.borrow_mut().push(msg)
    }

    pub fn schedule(&self, msg: Message, d: Duration) {
        self.0.borrow_mut().schedule(msg, d)
    }

    pub fn schedule_at(&self, msg: Message, ins: Instant) {
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
