use damnpacket::Message;
use damnpacket::MessageIsh;
use hooks::{Hooks,HookStorage};
use messagequeue::MessageQueue;
use std::collections::HashMap;
use event::{Event,EType};
use std::convert::TryFrom;
use std::rc::Rc;
use diesel::sqlite::SqliteConnection;

type Callback = fn(Message, MessageQueue, &mut HookStorage, &Rc<SqliteConnection>);

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

fn respond_ping(_: Message, mq: MessageQueue, _: &mut HookStorage, _: &Rc<SqliteConnection>) {
    mq.push(Message::from("pong\n\0"));
}

fn respond_damnserver(_: Message, mq: MessageQueue, _: &mut HookStorage, _: &Rc<SqliteConnection>) {
    mq.push(Message::from(concat!("login participle\npk=", env!("PK"), "\n\0")));
}

fn respond_login(msg: Message, mq: MessageQueue, _: &mut HookStorage, _: &Rc<SqliteConnection>) {
    match msg.get_attr(&b"e"[..]) {
        Some("ok") => {
            info!("Logged in successfully");
            info!("Joining chat:devintesting");
            mq.push(Message::from("join chat:devintesting\n\0"));
        },
        x => error!("Failed to log in: {:?}", x)
    };
}

fn respond_recv(msg: Message, mq: MessageQueue, h: &mut HookStorage, s: &Rc<SqliteConnection>) {
    if let Ok(ev) = Event::try_from((&msg, s.clone(), mq)) {
        let updates = match ev.ty {
            EType::Join => h.join_iter().flat_map(|cmd| {
                cmd(ev.clone())
            }).collect::<Hooks>(),
            EType::Part => {
                vec![]
            },
            _ => h.msg_iter().flat_map(|cmd| {
                cmd(ev.clone())
            }).collect::<Hooks>()
        };
        h.apply(updates);
    }
}
