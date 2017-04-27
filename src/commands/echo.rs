use commands::prelude::*;

pub fn echo(e: Event) -> Hooks {
    if e.sender == b"participle" || e.message.len() <= 6 {
        return vec![]
    }

    e.respond(&e.message[6..]);
    vec![]
}
