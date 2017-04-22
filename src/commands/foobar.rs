use commands::prelude::*;

pub fn foo(m: M) -> Command {
    box move |e| {
        e.respond("Disabling !foo and enabling !bar");
        vec![
            Hook::unregister(m),
            Hook::register("bar", bar),
        ]
    }
}

pub fn bar(m: M) -> Command {
    box move |e| {
        e.respond("Disabling !bar and enabling !foo");
        vec![
            Hook::unregister(m),
            Hook::register("foo", foo),
        ]
    }
}
