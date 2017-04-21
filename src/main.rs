#![feature(try_from)]

extern crate ansi_term;
extern crate bytes;
extern crate damnpacket;
extern crate futures;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate nom;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_timer;

use ansi_term::Colour;
use damnpacket::Message;
use futures::future::Future;
use futures::{Stream, Sink};
use std::io::BufRead;
use handler::Hooks;
use std::io;
use std::net::{SocketAddr,ToSocketAddrs};
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core,Handle};
use tokio_io::AsyncRead;
use env_logger::LogBuilder;
use std::env;

pub mod codec;
pub mod commands;
pub mod event;
pub mod handler;
pub mod messagequeue;

use codec::DamnCodec;
use handler::ACTIONS;
use messagequeue::MessageQueue;

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

fn dump(it: &damnpacket::Message, direction: bool) {
    let prefix = if direction {
        Colour::Fixed(11).paint("⟹  ")
    } else {
        Colour::Fixed(13).paint("⟸  ")
    };
    let mut output = vec![];
    it.pretty(&mut output).unwrap();
    let lines = std::io::BufReader::new(&output[..]);
    for line in lines.lines() {
        debug!("{} {}", prefix, line.unwrap());
    }
}

fn repeatedly(h: &Handle, addr: &SocketAddr) {
    let greeting = Message::from("dAmnClient 0.3\nagent=marsipan\n\0");
    let a2 = addr.clone();
    let h2 = h.clone();
    let mq = MessageQueue::new(&h);
    let mq2 = mq.clone();
    let hooks = Hooks::new();
    let hooks2 = hooks.clone();
    hooks.add_command("ping", Box::new(commands::cmd_ping));
    h.spawn(TcpStream::connect(&addr, &h).then(|res|
        match res {
            Ok(stream) => Ok(stream.framed(DamnCodec).split()),
            Err(e) => Err(MarsError::from(e))
        }
    ).and_then(|(tx, rx)|
        tx.send(greeting).and_then(|writer| {
            rx.and_then(move |item| {
                dump(&item, true);
                match ACTIONS.get(&item.name[..]) {
                    Some(f) => f(item, mq.clone(), hooks2.clone()),
                    _ => debug!("unknown message")
                };
                Ok(None)
            })
                .filter_map(|x|x)
                .select(mq2)
                .map(|item| { dump(&item, false); item })
                .forward(writer)
        })
    ).map(|_| ())
    .or_else(move |e| {
        warn!("Error during respond loop: {:?}", e);
        repeatedly(&h2, &a2);
        Ok(())
    }))
}

fn log_init() -> Result<(), log::SetLoggerError> {
    let mut builder = LogBuilder::new();

    if let Ok(s) = env::var("RUST_LOG") {
        builder.parse(&s);
    }

    fn pretty_level(l: log::LogLevel) -> &'static str {
        use log::LogLevel;
        match l {
            LogLevel::Error => "\x1b[31mERR\x1b[0m ",
            LogLevel::Warn => "\x1b[33mWRN\x1b[0m ",
            LogLevel::Info => "\x1b[34mINF\x1b[0m ",
            LogLevel::Debug => "\x1b[35mDBG\x1b[0m ",
            LogLevel::Trace => "\x1b[36mTRC\x1b[0m "
        }
    }

    builder.format(|record|
        format!("{}{}", pretty_level(record.level()), record.args())
    ).init()
}

fn main() {
    log_init().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "chat.deviantart.com:3900".to_socket_addrs().unwrap().next().unwrap();
    repeatedly(&handle, &addr);
    core.run(futures::future::empty::<(),()>()).unwrap();
}
