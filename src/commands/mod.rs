use hooks::{Command,Hook,Hooks};

pub mod about;
pub mod foobar;
pub mod ping;
pub mod prelude;

macro_rules! cmd {
    ($e:expr) => { |_| box $e as Command };
}

pub fn default_cmds() -> Hooks {
    vec![
        Hook::register("ping", cmd!(ping::ping)),
        Hook::register("about", cmd!(about::about)),
        Hook::register("foo", foobar::foo),
    ]
}
