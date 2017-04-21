use hooks::{Command,Hook,Updates};

pub mod ping;
pub mod prelude;

pub fn default_cmds() -> Updates {
    vec![
        Hook::register("ping", |_| Box::new(ping::ping) as Command),
    ]
}
