use hooks::{Command,Hook,Hooks};

mod about;
mod commands;
mod echo;
mod ping;
mod prelude;
mod sleep;
mod welcome;
mod wordwar;

macro_rules! cmd {
    ($e:expr) => { |_| box $e as Command };
}

macro_rules! cmds {
    (
        cmd => [$($e:expr => $f:expr),*],
        msg => [$($g:expr),*],
        join => [$($h:expr),*]
    ) => {
        static CMD_NAMES: &'static [&'static str] = &[$($e,)*];

        let mut v = vec![$(Hook::register($e, $f),)*];
        v.extend(vec![$(Hook::register_msg($g),)*]);
        v.extend(vec![$(Hook::join($h),)*]);
        v
    }
}

pub fn default_cmds() -> Hooks {
    cmds! {
        cmd => [ "about" => cmd!(about::about),
                 "commands" => |_| commands::commands(CMD_NAMES),
                 "echo" => cmd!(echo::echo),
                 "ping" => cmd!(ping::ping),
                 "sleep" => cmd!(sleep::sleep),
                 "wakeup" => cmd!(sleep::wakeup),
                 "welcome" => cmd!(welcome::welcome),
                 "ww" => cmd!(wordwar::wordwar) ],

        msg => [],

        join => [ cmd!(welcome::say_welcome) ]
    }
}
