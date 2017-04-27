use hooks::{Command,Hook,Hooks};

mod about;
mod echo;
mod foobar;
mod ping;
mod prelude;

macro_rules! cmd {
    ($e:expr) => { |_| box $e as Command };
}

pub fn default_cmds() -> Hooks {
    vec![
        Hook::register("ping", cmd!(ping::ping)),
        Hook::register("about", cmd!(about::about)),
        Hook::register("echo", cmd!(echo::echo)),
    ]
}
