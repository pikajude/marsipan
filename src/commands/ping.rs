use commands::prelude::*;
use std::time::Instant;

pub fn ping(e: Event) -> Updates {
    e.respond("\u{1f514}?");
    let t = Instant::now();
    vec![Hook::register_msg(|m| Box::new(move |e| {
        if e.message == "\u{1f514}?" {
            let diff = Instant::now() - t;
            let ms = (diff.subsec_nanos().checked_div(1000000).unwrap() as u64)
                + diff.as_secs() * 1000;
            e.respond(format!("\u{1f514}! ({}ms)", ms));
            vec![Hook::unregister(m)]
        } else {
            vec![]
        }
    }))]
}
