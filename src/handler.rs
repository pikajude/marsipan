use damnpacket::Message;
use damnpacket::MessageIsh;
use hooks::{Hooks,Updates};
use messagequeue::MessageQueue;
use std::collections::HashMap;
use event::{Event,EType};
use std::convert::TryFrom;

type Callback = fn(Message, MessageQueue, &mut Hooks);

lazy_static! {
    pub static ref ACTIONS: HashMap<&'static [u8], Callback> = {
        let mut m = HashMap::new();
        m.insert(&b"dAmnServer"[..], respond_damnserver as Callback);
        m.insert(&b"login"[..], respond_login as Callback);
        m.insert(&b"ping"[..], respond_ping as Callback);
        m.insert(&b"recv"[..], respond_recv as Callback);
        m
    };
}

fn respond_ping(_: Message, mq: MessageQueue, _: &mut Hooks) {
    mq.push(Message::from("pong\n\0"));
}

fn respond_damnserver(_: Message, mq: MessageQueue, _: &mut Hooks) {
    mq.push(Message::from(concat!("login participle\npk=", env!("PK"), "\n\0")));
}

fn respond_login(msg: Message, mq: MessageQueue, _: &mut Hooks) {
    match msg.get_attr(&b"e"[..]) {
        Some("ok") => {
            info!("Logged in successfully");
            info!("Joining chat:devintesting");
            mq.push(Message::from("join chat:devintesting\n\0"));
        },
        x => error!("Failed to log in: {:?}", x)
    };
}

fn respond_recv(msg: Message, mq: MessageQueue, h: &mut Hooks) {
    if let Ok(ev) = Event::try_from((&msg, mq)) {
        let updates = match ev.ty {
            EType::Join => h.join_iter().flat_map(|cmd| {
                cmd(ev.clone())
            }).collect::<Updates>(),
            EType::Part => {
                vec![]
            },
            _ => h.msg_iter().flat_map(|cmd| {
                cmd(ev.clone())
            }).collect::<Updates>()
        };
        h.apply(updates);
    }
}
