use hooks::{Command,Hook,Hooks};

mod about;
mod commands;
mod echo;
mod ping;
mod prelude;

macro_rules! cmd {
    ($e:expr) => { |_| box $e as Command };
}

macro_rules! cmds {
    ($($e:expr => $f:expr),*) => {
        static CMD_NAMES: &'static [&'static str] = &[$($e,)*];

        vec![$(Hook::register($e, $f),)*]
    }
}

pub fn default_cmds() -> Hooks {
    cmds! {
        "about" => cmd!(about::about),
        "commands" => |_| commands::commands(CMD_NAMES),
        "echo" => cmd!(echo::echo),
        "ping" => cmd!(ping::ping)
    }
}
