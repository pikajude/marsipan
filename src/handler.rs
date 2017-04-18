use damnpacket::Message;
use damnpacket::MessageIsh;
use messagequeue::MessageQueue;
use std::collections::HashMap;

type Callback = fn(Message, MessageQueue);

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

fn respond_ping(_: Message, mq: MessageQueue) {
    mq.push(Message::from("pong\n\0"));
}

fn respond_damnserver(_: Message, mq: MessageQueue) {
    mq.push(Message::from(concat!("login participle\npk=", env!("PK"), "\n\0")));
}

fn respond_login(msg: Message, mq: MessageQueue) {
    match msg.get_attr(&b"e"[..]) {
        Some("ok") => {
            info!("Logged in successfully");
            info!("Joining chat:devintesting");
            mq.push(Message::from("join chat:devintesting\n\0"));
        },
        x => error!("Failed to log in: {:?}", x)
    };
}

fn respond_recv(msg: Message, mq: MessageQueue) {
    for sub in msg.submessage().into_iter() {
        match sub.name.as_ref().map(|x|x.as_slice()) {
            Some(b"msg") | Some(b"action") => debug!("Received a message: {}", sub.body_().to_string()),
            _ => debug!("Unknown subtype")
        }
    }
}
