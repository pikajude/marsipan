use damnpacket::Message;
use futures::future;
use futures::Future;
use messagequeue::MQ;
use std::collections::HashMap;
use MarsError;

type Response = Box<Future<Item=Option<Message>, Error=MarsError>>;

type Callback = fn(Message, MQ) -> Response;

lazy_static! {
    pub static ref ACTIONS: HashMap<&'static [u8], Callback> = {
        let mut m = HashMap::new();
        m.insert(&b"dAmnServer"[..], respond_damnserver as Callback);
        m.insert(&b"login"[..], respond_login as Callback);
        m
    };
}

pub fn wrap(x: Option<Message>) -> Response {
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

fn respond_login(msg: Message, _: MQ) -> Response {
    match msg.get_attr(&b"e"[..]) {
        Some("ok") => info!("Logged in successfully"),
        x => error!("Failed to log in: {:?}", x)
    };
    wrap(None)
}
