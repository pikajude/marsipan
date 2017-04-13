#![allow(non_snake_case)]

extern crate ansi_term;
extern crate bytes;
extern crate damnpacket;
extern crate futures;
extern crate nom;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;

#[macro_use]
extern crate lazy_static;

use ansi_term::Colour;
use bytes::BytesMut;
use damnpacket::Message;
use futures::future;
use futures::future::Future;
use futures::{Stream, Sink};
use std::io;
use std::collections::HashMap;
use std::io::BufRead;
use std::net::{SocketAddr,ToSocketAddrs};
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core,Handle};
use tokio_io::AsyncRead;
use tokio_io::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum MarsError {
    Io(io::Error),
    Parse(nom::ErrorKind),
    Fut(futures::sync::mpsc::SendError<Message>),
    Nul,
}

impl From<()> for MarsError {
    fn from(x: ()) -> Self {
        MarsError::Nul
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

type Response = Box<Future<Item=Option<Message>, Error=MarsError>>;

type Callback = fn(Message) -> Response;

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

fn respond_damnserver(_: Message) -> Response {
    wrap(Some(Message {
        name: b"login".to_vec(),
        argument: Some(b"participle".to_vec()),
        attrs: vec![(b"pk".to_vec(), String::from(env!("PK")))].into_iter().collect(),
        body: None,
    }))
}

fn respond_login(msg: Message) -> Response {
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
    println!("Connecting");
    h.spawn(TcpStream::connect(&addr, &h).then(|res|
        match res {
            Ok(stream) => Ok(stream.framed(DamnCodec).split()),
            Err(e) => Err(MarsError::from(e))
        }
    ).and_then(|(tx, rx)| {
        let (chansend, chanrecv) = futures::sync::mpsc::channel(16);
        tx.send(greeting).and_then(|writer| {
            rx.and_then(|item| {
                dump(&item, true);
                match ACTIONS.get(item.name.as_slice()) {
                    Some(f) => f(item),
                    _ => {
                        println!("unknown message");
                        Box::new(future::ok(None))
                    }
                }
            }).filter_map(|x|x)
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
