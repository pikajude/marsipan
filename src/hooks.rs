use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize,ATOMIC_USIZE_INIT,Ordering};
use std::collections::hash_map::Values;
use event::Event;

static TRIGGERS: [&'static str; 2] = ["!", "participle: "];

pub type Command = Box<Fn(Event) -> Hooks + Send>;

pub struct HookStorage {
    msg: HashMap<M, Command>,
    join: HashMap<J, Command>,
}

impl HookStorage {
    pub fn new() -> Self {
        HookStorage {
            msg: HashMap::new(),
            join: HashMap::new(),
        }
    }

    fn add_command(&mut self, u: M, s: &'static str, cb: Command) -> M {
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

        self.msg.insert(u, box move |ev|
            if matches(&ev, s) {
                cb(ev)
            } else {
                vec![]
            }
        );
        u
    }

    pub fn join_iter<'a>(&'a self) -> Values<'a, J, Command> {
        self.join.values()
    }

    pub fn msg_iter<'a>(&'a self) -> Values<'a, M, Command> {
        self.msg.values()
    }

    pub fn apply(&mut self, updates: Hooks) {
        for up in updates.into_iter() {
            match up {
                Hook::AddMessage(m,c) => {self.msg.insert(m,c);},
                Hook::AddCommand(m,s,c) => {self.add_command(m,s,c);}
                Hook::AddJoin(j,c) => {self.join.insert(j,c);},
                Hook::DropMessage(m) => {self.msg.remove(&m);},
                Hook::DropJoin(j) => {self.join.remove(&j);},
            }
        }
    }
}

static UNIQUE: AtomicUsize = ATOMIC_USIZE_INIT;

fn new_unique() -> usize {
    UNIQUE.fetch_add(1, Ordering::SeqCst)
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct M(usize);
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct J(usize);

impl M {
    pub fn next() -> Self {
        M(new_unique())
    }
}

impl J {
    pub fn next() -> Self {
        J(new_unique())
    }
}

pub enum Hook {
    AddMessage(M, Command),
    AddCommand(M, &'static str, Command),
    AddJoin(J, Command),
    DropMessage(M),
    DropJoin(J),
}

pub type Hooks = Vec<Hook>;

impl Hook {
    pub fn register<F>(s: &'static str, f: F) -> Self
        where F: FnOnce(M) -> Command {
        let m = M::next();
        Hook::AddCommand(m, s, f(m))
    }

    pub fn register_msg<F>(f: F) -> Self
        where F: FnOnce(M) -> Command {
        let m = M::next();
        Hook::AddMessage(m, f(m))
    }

    pub fn unregister(m: M) -> Self {
        Hook::DropMessage(m)
    }
}
