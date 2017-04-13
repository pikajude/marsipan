#![allow(non_snake_case)]

extern crate ansi_term;
extern crate bytes;
extern crate damnpacket;
extern crate futures;
extern crate nom;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_timer;

#[macro_use]
extern crate lazy_static;

use ansi_term::Colour;
use bytes::BytesMut;
use damnpacket::Message;
use futures::Async;
use futures::future;
use futures::future::Future;
use std::ops::DerefMut;
use futures::{Stream, Sink};
use std::collections::{BinaryHeap,HashMap};
use std::io;
use std::io::BufRead;
use std::net::{SocketAddr,ToSocketAddrs};
use std::cmp::Ordering;
use std::time::{Duration,Instant};
use std::sync::RwLock;
use tokio_core::net::TcpStream;
use std::rc::Rc;
use std::cell::RefCell;
use tokio_core::reactor::{Core,Handle};
use tokio_core::reactor::Timeout;
use tokio_io::AsyncRead;
use tokio_io::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum MarsError {
    Io(io::Error),
    Parse(nom::ErrorKind),
    Fut(futures::sync::mpsc::SendError<Message>),
}

impl From<()> for MarsError {
    fn from(_: ()) -> Self {
        unreachable!("no () -> MarsError")
    }
}

impl From<io::Error> for MarsError {
    fn from(e: io::Error) -> Self {
        MarsError::Io(e)
    }
}

impl From<nom::ErrorKind> for MarsError {
    fn from(e: nom::ErrorKind) -> Self {
        MarsError::Parse(e)
    }
}

impl From<futures::sync::mpsc::SendError<damnpacket::Message>> for MarsError {
    fn from(e: futures::sync::mpsc::SendError<damnpacket::Message>) -> Self {
        MarsError::Fut(e)
    }
}

#[derive(Debug)]
struct DamnCodec;

impl Decoder for DamnCodec {
    type Item = Message;
    type Error = MarsError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(i) = buf.iter().position(|&b| b == b'\0') {
            let line = buf.split_to(i + 1);
            match damnpacket::parse(&line[..]) {
                Ok(msg) => Ok(Some(msg)),
                Err(e) => Err(MarsError::from(e)),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for DamnCodec {
    type Item = Message;
    type Error = MarsError;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.extend(msg.as_bytes());
        Ok(())
    }
}

#[derive(Debug)]
struct CmpFst<A,B>(A,B);

impl<A,B> CmpFst<A,B> {
    fn fst(&self) -> &A {
        &self.0
    }

    fn snd(self) -> B {
        self.1
    }
}

impl<A,B> PartialEq for CmpFst<A,B> where A: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A,B> Eq for CmpFst<A,B> where A: Eq {

}

impl<A,B> PartialOrd for CmpFst<A,B> where A: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(|x|x.reverse())
    }
}

impl<A,B> Ord for CmpFst<A,B> where A: Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

struct MessageQueue {
    heap: BinaryHeap<CmpFst<Instant, Message>>,
    timeout: Option<Timeout>,
    handle: Handle
}

#[derive(Clone)]
struct MQ(Rc<RefCell<MessageQueue>>);

impl MessageQueue {
    fn new(h: &Handle) -> Self {
        MessageQueue {
            heap: BinaryHeap::new(),
            timeout: None,
            handle: h.clone()
        }
    }

    fn schedule(&mut self, msg: Message, d: Duration) {
        self.heap.push(CmpFst(Instant::now() + d, msg));
        self.reschedule();
    }

    fn reschedule(&mut self) {
        if let Some(soonest) = self.heap.peek() {
            self.timeout = Some(
                Timeout::new(*soonest.fst() - Instant::now(), &self.handle)
                    .expect("Timeout::new should never fail")
            );
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
                    Ok(Async::Ready(Some(self.heap.pop().expect("Invariant: timeout with empty heap").snd())))
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

impl MQ {
    fn schedule(self, msg: Message, d: Duration) {
        self.0.borrow_mut().schedule(msg, d)
    }
}

impl Stream for MQ {
    type Item = damnpacket::Message;
    type Error = MarsError;

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        self.0.borrow_mut().poll()
    }
}

type Response = Box<Future<Item=Option<Message>, Error=MarsError>>;

type Callback = fn(Message, MQ) -> Response;

lazy_static! {
    static ref ACTIONS: HashMap<&'static [u8], Callback> = {
        let mut m = HashMap::new();
        m.insert(&b"dAmnServer"[..], respond_damnserver as Callback);
        m.insert(&b"login"[..], respond_login as Callback);
        m
    };
}

fn wrap(x: Option<Message>) -> Response {
    Box::new(future::ok(x))
}

fn respond_damnserver(_: Message, _: MQ) -> Response {
    wrap(Some(Message {
        name: b"login".to_vec(),
        argument: Some(b"participle".to_vec()),
        attrs: vec![(b"pk".to_vec(), String::from(env!("PK")))].into_iter().collect(),
        body: None,
    }))
}

fn respond_login(msg: Message, mq: MQ) -> Response {
    mq.schedule(Message::from(&b"login participle\ngarbage goes here\n\0"[..]), Duration::new(10, 0));
    match msg.get_attr(&b"e"[..]) {
        Some("ok") => println!("success"),
        x => panic!("Failed to log in: {:?}", x)
    };
    wrap(None)
}

fn dump(it: &damnpacket::Message, direction: bool) {
    let prefix = if direction {
        Colour::Fixed(11).paint(">>>")
    } else {
        Colour::Fixed(13).paint("<<<")
    };
    let mut output = vec![];
    it.pretty(&mut output).unwrap();
    let lines = std::io::BufReader::new(&output[..]);
    for line in lines.lines() {
        println!("{} {}", prefix, line.unwrap());
    }
    println!("");
}

fn repeatedly(h: &Handle, addr: &SocketAddr) {
    let greeting = Message::from(&b"dAmnClient 0.3\nagent=marsipan\n\0"[..]);
    let a2 = addr.clone();
    let h2 = h.clone();
    let mq = MQ(Rc::new(RefCell::new(MessageQueue::new(&h))));
    let mq2 = mq.clone();
    h.spawn(TcpStream::connect(&addr, &h).then(|res|
        match res {
            Ok(stream) => Ok(stream.framed(DamnCodec).split()),
            Err(e) => Err(MarsError::from(e))
        }
    ).and_then(|(tx, rx)| {
        let (_chansend, chanrecv) = futures::sync::mpsc::channel(16);
        tx.send(greeting).and_then(|writer| {
            rx.and_then(move |item| {
                dump(&item, true);
                match ACTIONS.get(item.name.as_slice()) {
                    Some(f) => f(item, mq.clone()),
                    _ => {
                        println!("unknown message");
                        wrap(None)
                    }
                }
            })
                .filter_map(|x|x)
                .select(mq2)
                .select(chanrecv.map_err(|x|MarsError::from(x)))
                .map(|item| { dump(&item, false); item })
                .forward(writer)
        })
    }).map(|_| ())
    .or_else(move |e| {
        println!("An error: {:?}", e);
        repeatedly(&h2, &a2);
        Ok(())
    }))
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "chat.deviantart.com:3900".to_socket_addrs().unwrap().next().unwrap();
    repeatedly(&handle, &addr);
    core.run(futures::future::empty::<(),()>()).unwrap();
}
