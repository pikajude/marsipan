use damnpacket::{Message,MessageBody};
use std::convert::TryFrom;
use std::collections::HashMap;
use messagequeue::MessageQueue;
use commands::Command;
use handler::{Hooks,M};
use std::ops::Deref;

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
    pub hooks: Hooks,
}

impl<'a> TryFrom<(&'a Message, MessageQueue, Hooks)> for Event {
    type Error = ();

    fn try_from(arg: (&'a Message, MessageQueue, Hooks)) -> Result<Self, ()> {
        let (msg, mq, hs) = arg;
        let chatroom = msg.argument.clone();
        for sub in msg.submessage().into_iter() {
            let sender = sub.argument.clone();
            if sender == Some(b"participle".to_vec()) {
                return Err(())
            }
            return match sub.name.as_ref().map(|x|x.as_slice()) {
                Some(b"msg") => Ok(Event {
                    ty: EType::Message,
                    chatroom: chatroom.expect("invariant: recv msg, no chatroom"),
                    sender: sender.expect("invariant: recv msg, no sender"),
                    message: sub.body.map(|x|x.to_string()).unwrap_or("".to_string()),
                    mq: mq,
                    hooks: hs,
                }),
                Some(b"action") => Ok(Event {
                    ty: EType::Action,
                    chatroom: chatroom.expect("invariant: recv action, no chatroom"),
                    sender: sender.expect("invariant: recv action, no sender"),
                    message: sub.body.map(|x|x.to_string()).unwrap_or("".to_string()),
                    mq: mq,
                    hooks: hs,
                }),
                Some(b"join") => Ok(Event {
                    ty: EType::Join,
                    chatroom: chatroom.expect("invariant: recv join, no chatroom"),
                    sender: sender.expect("invariant: recv join, no sender"),
                    message: "".to_string(),
                    mq: mq,
                    hooks: hs,
                }),
                Some(b"part") => Ok(Event {
                    ty: EType::Part,
                    chatroom: chatroom.expect("invariant: recv part, no chatroom"),
                    sender: sender.expect("invariant: recv part, no sender"),
                    message: "".to_string(),
                    mq: mq,
                    hooks: hs,
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

    pub fn add_msg(&self, c: Command) -> M {
        self.hooks.clone().add_msg(c)
    }
}
