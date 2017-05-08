use commands::prelude::*;
use std::time::Instant;

pub fn ping(e: &Event) -> Hooks {
    e.respond("\u{1f514}?");
    let t = Instant::now();
    vec![Hook::register_msg(|m| box move |e| {
        if e.message == "\u{1f514}?" {
            let diff = Instant::now() - t;
            let ms = (diff.subsec_nanos() as u64 / 1000000)
                + diff.as_secs() * 1000;
            e.respond(format!("\u{1f514}! ({}ms)", ms));
            return vec![Hook::unregister(m)];
        }

        vec![]
    })]
}
