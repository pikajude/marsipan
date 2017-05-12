use commands::prelude::*;

pub fn commands(names: &'static [&'static str]) -> Command {
    box move |e| {
        let mut n = names.to_vec();
        n.sort();
        e.respond_highlight(format!("Commands are: {}", n.join(", ")));
        vec![]
    }
}
