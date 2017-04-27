use damnpacket::{Message,MessageBody,MessageIsh};
use std::convert::TryFrom;
use std::collections::HashMap;
use messagequeue::MessageQueue;

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

    mq: MessageQueue,
    // updates: Vec<Update>,
}

impl<'a> TryFrom<(&'a Message, MessageQueue)> for Event {
    type Error = ();

    fn try_from(arg: (&'a Message, MessageQueue)) -> Result<Self, ()> {
        let (msg, mq) = arg;
        let chatroom = msg.argument.clone();
        for sub in msg.submessage().into_iter() {
            return match sub.name.as_ref().map(|x|x.as_slice()) {
                Some(b"msg") => Ok(Event {
                    ty: EType::Message,
                    chatroom: chatroom.expect("invariant: recv msg, no chatroom"),
                    sender: sub.get_attr("from").expect("invariant: recv msg, no sender").as_bytes().to_vec(),
                    message: sub.body.map(|x|x.to_string()).unwrap_or("".to_string()),
                    mq: mq,
                }),
                Some(b"action") => Ok(Event {
                    ty: EType::Action,
                    chatroom: chatroom.expect("invariant: recv action, no chatroom"),
                    sender: sub.get_attr("from").expect("invariant: recv action, no sender").as_bytes().to_vec(),
                    message: sub.body.map(|x|x.to_string()).unwrap_or("".to_string()),
                    mq: mq,
                }),
                Some(b"join") => Ok(Event {
                    ty: EType::Join,
                    chatroom: chatroom.expect("invariant: recv join, no chatroom"),
                    sender: sub.argument.clone().expect("invariant: recv join, no sender"),
                    message: "".to_string(),
                    mq: mq,
                }),
                Some(b"part") => Ok(Event {
                    ty: EType::Part,
                    chatroom: chatroom.expect("invariant: recv part, no chatroom"),
                    sender: sub.argument.clone().expect("invariant: recv part, no sender"),
                    message: "".to_string(),
                    mq: mq,
                }),
                _ => Err(())
            }
        }
        Err(())
    }
}

impl Event {
    pub fn respond<S>(&self, msg: S)
        where S: Into<String> {
        self.mq.clone().push(Message {
            name: b"send".to_vec(),
            argument: Some(self.chatroom.clone()),
            attrs: HashMap::new(),
            body: Some(MessageBody::from(format!("msg main\n\n{}\0", msg.into())))
        })
    }

    pub fn respond_highlight<S>(&self, msg: S)
        where S: Into<String> {
        self.respond(format!("{}: {}",
            // dA enforces that names are ascii so this is OK
            ::std::str::from_utf8(self.sender.as_slice()).unwrap(),
            msg.into()))
    }
}
