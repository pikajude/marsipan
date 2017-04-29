use commands::prelude::*;

pub fn commands(names: &'static [&'static str]) -> Command {
    box move |e| {
        e.respond_highlight(format!("Commands are: {}", names.join(", ")));
        vec![]
    }
}
