use damnpacket::Message;
use damnpacket::MessageIsh;
use messagequeue::MessageQueue;
use std::collections::HashMap;
use std::collections::hash_map::Values;
use event::{Event,EType};
use std::sync::atomic::{AtomicUsize,ATOMIC_USIZE_INIT,Ordering};
use std::convert::TryFrom;
use std::cell::RefCell;
use std::rc::Rc;
use commands;
use commands::Command;

static TRIGGERS: [&'static str; 2] = ["!", "participle: "];

type Callback = fn(Message, MessageQueue, Hooks);

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

static UNIQUE: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct M(usize);
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct J(usize);

fn new_unique() -> usize {
    UNIQUE.fetch_add(1, Ordering::SeqCst)
}

struct H {
    msg: HashMap<M, Command>,
    join: HashMap<J, Command>,
}

#[derive(Clone)]
pub struct Hooks(Rc<RefCell<H>>);

impl Hooks {
    pub fn new() -> Self {
        Hooks(Rc::new(RefCell::new(H {
            msg: HashMap::new(),
            join: HashMap::new(),
        })))
    }

    pub fn add_command(self, s: &'static str, cb: Command) -> M {
        fn matches(ev: &Event, cmd: &str) -> bool {
            for t in TRIGGERS.iter() {
                if ev.message.starts_with(t) {
                    if (&ev.message[t.len()..]).starts_with(cmd) {
                        return true
                    }
                }
            }
            false
        }

        let u = M(new_unique());
        self.0.borrow_mut().msg.insert(u, Box::new(move |ev|
            if matches(&ev, s) {
                cb(ev)
            }
        ));
        u
    }

    pub fn add_msg(self, c: Command) -> M {
        let u = M(new_unique());
        self.0.borrow_mut().msg.insert(u, c);
        u
    }

    fn add_join(self, c: Command) -> J {
        let u = J(new_unique());
        self.0.borrow_mut().join.insert(u, c);
        u
    }

    fn remove(self, u: M) {
        self.0.borrow_mut().msg.remove(&u);
    }

    fn remove_join(self, u: J) {
        self.0.borrow_mut().join.remove(&u);
    }
}

fn respond_ping(_: Message, mq: MessageQueue, _: Hooks) {
    mq.push(Message::from("pong\n\0"));
}

fn respond_damnserver(_: Message, mq: MessageQueue, _: Hooks) {
    mq.push(Message::from(concat!("login participle\npk=", env!("PK"), "\n\0")));
}

fn respond_login(msg: Message, mq: MessageQueue, _: Hooks) {
    match msg.get_attr(&b"e"[..]) {
        Some("ok") => {
            info!("Logged in successfully");
            info!("Joining chat:devintesting");
            mq.push(Message::from("join chat:devintesting\n\0"));
        },
        x => error!("Failed to log in: {:?}", x)
    };
}

fn respond_recv(msg: Message, mq: MessageQueue, h: Hooks) {
    if let Ok(ev) = Event::try_from((&msg, mq, h.clone())) {
        match ev.ty {
            EType::Join => for cmd in h.0.borrow().join.values() {
                cmd(ev.clone())
            },
            EType::Part => do_part_hook(ev),
            _ => for cmd in h.0.borrow().msg.values() {
                cmd(ev.clone())
            }
        }
    }
}

fn do_join_hook(_: Event) {  }
fn do_part_hook(_: Event) {  }
