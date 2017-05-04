use damnpacket::{Message,MessageBody,MessageIsh};
use diesel::ExecuteDsl;
use diesel::sqlite::SqliteConnection;
use messagequeue::MessageQueue;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::{Duration,Instant};
use std::str::SplitWhitespace;
use std::rc::Rc;
use diesel::LoadDsl;

#[derive(Debug, Clone)]
pub enum EType {
    Join, Part, Message, Action
}

#[derive(Clone)]
pub struct Event {
    pub ty: EType,
    pub chatroom: Vec<u8>,
    pub sender: Vec<u8>,
    pub message: String,

    connection: Rc<SqliteConnection>,

    mq: MessageQueue,
}

impl<'a> TryFrom<(&'a Message, Rc<SqliteConnection>, MessageQueue)> for Event {
    type Error = ();

    fn try_from(arg: (&'a Message, Rc<SqliteConnection>, MessageQueue)) -> Result<Self, ()> {
        let (msg, conn, mq) = arg;
        let chatroom = msg.argument.clone();
        for sub in msg.submessage().into_iter() {
            return match sub.name.as_ref().map(|x|x.as_slice()) {
                Some(b"msg") => Ok(Event {
                    ty: EType::Message,
                    chatroom: chatroom.expect("invariant: recv msg, no chatroom"),
                    sender: sub.get_attr("from").expect("invariant: recv msg, no sender").as_bytes().to_vec(),
                    message: sub.body.map(|x|x.to_string()).unwrap_or("".to_string()),
                    connection: conn,
                    mq: mq,
                }),
                Some(b"action") => Ok(Event {
                    ty: EType::Action,
                    chatroom: chatroom.expect("invariant: recv action, no chatroom"),
                    sender: sub.get_attr("from").expect("invariant: recv action, no sender").as_bytes().to_vec(),
                    message: sub.body.map(|x|x.to_string()).unwrap_or("".to_string()),
                    connection: conn,
                    mq: mq,
                }),
                Some(b"join") => Ok(Event {
                    ty: EType::Join,
                    chatroom: chatroom.expect("invariant: recv join, no chatroom"),
                    sender: sub.argument.clone().expect("invariant: recv join, no sender"),
                    message: "".to_string(),
                    connection: conn,
                    mq: mq,
                }),
                Some(b"part") => Ok(Event {
                    ty: EType::Part,
                    chatroom: chatroom.expect("invariant: recv part, no chatroom"),
                    sender: sub.argument.clone().expect("invariant: recv part, no sender"),
                    message: "".to_string(),
                    connection: conn,
                    mq: mq,
                }),
                _ => Err(())
            }
        }
        Err(())
    }
}

pub fn word<'a>(s: &'a str) -> (&'a str, &'a str) {
    match s.split_at(s.find(' ').unwrap_or(s.len())) {
        (x, y) => (x, if y.len() > 0 { &y[1..] } else { y })
    }
}

impl Event {
    fn mk<S>(&self, msg: S) -> Message
        where S: Into<String> {
        Message {
            name: b"send".to_vec(),
            argument: Some(self.chatroom.clone()),
            attrs: HashMap::new(),
            body: Some(MessageBody::from(format!("msg main\n\n{}\0", msg.into())))
        }
    }

    pub fn content<'a>(&'a self) -> &'a str {
        word(&self.message).1
    }

    pub fn cancel(&self, i: Instant) -> Option<Message> {
        self.mq.clone().unschedule(i)
    }

    pub fn respond<S>(&self, msg: S) -> Instant
        where S: Into<String> {
        self.mq.clone().push(self.mk(msg))
    }

    pub fn respond_in<S>(&self, msg: S, d: Duration) -> Instant
        where S: Into<String> {
        self.mq.clone().schedule(self.mk(msg), d)
    }

    pub fn respond_at<S>(&self, msg: S, i: Instant) -> Instant
        where S: Into<String> {
        self.mq.clone().schedule_at(self.mk(msg), i)
    }

    pub fn respond_highlight<S>(&self, msg: S) -> Instant
        where S: Into<String> {
        self.respond(format!("{}: {}",
            // dA enforces that names are ascii so this is OK
            ::std::str::from_utf8(self.sender.as_slice()).unwrap(),
            msg.into()))
    }

    pub fn load<T,U>(&self, x: T) -> Vec<U>
        where T: LoadDsl<SqliteConnection> + ::std::fmt::Debug,
              U: ::diesel::Queryable<T::SqlType,::diesel::sqlite::Sqlite>,
              ::diesel::sqlite::Sqlite: ::diesel::types::HasSqlType<T::SqlType> {
        info!("diesel: {:?}", x);
        x.load(&self.connection).expect("Unable to load SQL")
    }

    pub fn execute<T>(&self, x: T) -> usize
        where T: ExecuteDsl<SqliteConnection> + ::std::fmt::Debug {
        info!("diesel: {:?}", x);
        x.execute(&self.connection).expect("Unable to insert SQL")
    }
}
