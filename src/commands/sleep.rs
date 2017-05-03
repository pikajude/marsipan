use commands::prelude::*;

static mut N: Option<Instant> = None;

pub fn wakeup(e: Event) -> Hooks {
    if let Some(i) = unsafe {
        if let Some(n) = N {
            N = None;
            Some(n)
        } else {
            None
        }
    } {
        e.cancel(i);
        e.respond("Ok, I'm awake!");
    } else {
        e.respond("I wasn't sleeping!");
    }
    vec![]
}

pub fn sleep(e: Event) -> Hooks {
    if e.message.len() < 7 {
        return vec![];
    }
    match (&e.message[7..]).parse() {
        Ok(i) => {
            e.respond(format!("Sleeping for {} seconds. ZZZzzz...", i));
            let at = e.respond_in("Waking up!", Duration::new(i, 0));
            unsafe {
                N = Some(at);
            }
        },
        Err(_) => {
            e.respond("That doesn't look like a number.");
        }
    };
    vec![]
}
