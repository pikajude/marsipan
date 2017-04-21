use event::Event;

pub type Command = Box<Fn(Event) + Send>;

pub fn cmd_ping(e: Event) {
    e.add_msg(Box::new(|e|{
        debug!("a hooked message: {:?}", e.ty)
    }));
    e.respond("🔔!")
}
