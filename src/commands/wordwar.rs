use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use chrono::Timelike;
use commands::prelude::*;
use nom::digit;
use state::Storage;
use std::collections::{HashMap,HashSet};
use std::time::Duration as StdDuration;
use std::sync::{Mutex,MutexGuard};

fn until(other: DateTime<Local>) -> Option<StdDuration> {
    let d = other.signed_duration_since(Local::now());
    if d < Duration::zero() {
        return None
    }
    let nanos_only = d - Duration::seconds(d.num_seconds());
    Some(StdDuration::new(d.num_seconds() as u64, nanos_only.num_nanoseconds().unwrap() as u32))
}

static WARS: Storage<Mutex<HashMap<W, War>>> = Storage::new();

fn wars<'a>() -> MutexGuard<'a, HashMap<W, War>> {
    WARS.get().lock().unwrap()
}

named!(dec<u32>, map_res!(map_res!(digit, ::std::str::from_utf8), ::std::str::FromStr::from_str));

named!(parse_ww<(u32,u32)>, do_parse!(
    tag!(":") >>
    min: dec >>
    tag!(" for ") >>
    dur: dec >>
    (min, dur)
));

#[derive(Clone,Debug)]
struct War {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    start_msg: Option<Instant>,
    end_msg: Option<Instant>,
    participants: HashSet<String>,
    starter: String,
}

impl War {
    fn parse(bytes: &[u8]) -> Result<(DateTime<Local>, DateTime<Local>), String> {
        let (at, dur) = parse_ww(bytes).to_full_result()
            .map_err(|_|format!("Usage: !ww at :<b>time</b> for <b>minutes</b>"))?;
        if dur > 59 {
            return Err("Too many minutes.".to_string())
        }
        let current_time = Local::now();
        let start_time = if current_time.minute() >= at {
            current_time + Duration::hours(1)
        } else {
            current_time
        }.with_minute(at).and_then(|m|m.with_second(0)).ok_or("math error")?;
        Ok((start_time, start_time + Duration::minutes(dur as i64)))
    }

    fn register_msgs(&mut self, e: &Event) {
        self.cancel(e);
        let start = until(self.start_time).map(|t| {
            e.respond_in(format!(
                    "{}: <b>START WRITING!</b>",
                    self.participants.clone().into_iter().collect::<Vec<_>>().join(", "))
                , t)
        });
        let end = until(self.end_time).map(|t| {
            e.respond_in(format!(
                    "{}: <b>STOP WRITING!</b>",
                    self.participants.clone().into_iter().collect::<Vec<_>>().join(", "))
                , t)
        });
        self.start_msg = start;
        self.end_msg = end;
    }

    fn cancel(&self, e: &Event) {
        self.start_msg.map(|t|e.cancel(t));
        self.end_msg.map(|t|e.cancel(t));
    }
}

pub fn wordwar(e: &Event) -> Hooks {
    match word(e.content()) {
        ("at", rest) => wordwar_at(e, rest),
        ("cancel", id) => wordwar_cancel(e, id),
        ("list", _) => wordwar_list(e),
        x => { e.respond_highlight(format!("{:?}", x)); vec![] }
    }
}

fn wordwar_cancel(e: &Event, id: &str) -> Hooks {
    match id.parse() {
        Ok(h) => {
            let mut wars_guard = wars();
            match wars_guard.get(&h).cloned() {
                Some(w) => if w.starter == string!(e.sender) {
                    let war = wars_guard.remove(&h).unwrap();
                    war.cancel(&e);
                    e.respond_highlight(format!("Canceled war #{}.", h))
                } else {
                    e.respond_highlight("That's not yours.")
                },
                None => e.respond_highlight("No war with that ID found.")
            }
        },
        Err(_) => e.respond_highlight("That doesn't look like a war ID."),
    };
    vec![]
}

fn wordwar_list(e: &Event) -> Hooks {
    let mut response = "<ul>".to_string();
    for (k, v) in wars().iter() {
        response.push_str(&format!(
            "<li>#{id} (<b>{starter}</b>)<br><code>:{start} [===.........] :{end}</code></li>",
            id = k.un(),
            starter = v.starter,
            start = v.start_time.format("%M").to_string(),
            end = v.end_time.format("%M").to_string()));
    }
    response.push_str("</ul>");
    e.respond(response);

    vec![]
}

fn wordwar_at(e: &Event, rest: &str) -> Hooks {
    let res = War::parse(rest.as_bytes());
    match res {
        Ok((start_instant, end_instant)) => {
            let w = W::next();

            e.respond_highlight(format!("Scheduled war with ID #{}.", w));

            let w2 = w.clone();

            let mut new_war = War {
                start_time: start_instant,
                end_time: end_instant,
                start_msg: None,
                end_msg: None,
                participants: {
                    let mut h = HashSet::new();
                    h.insert(string!(e.sender));
                    h
                },
                starter: string!(e.sender),
            };
            new_war.register_msgs(&e);

            let start_cloned = new_war.start_msg.clone().unwrap();
            let end_cloned = new_war.end_msg.clone().unwrap();

            wars().insert(w, new_war);

            return vec![Hook::register("in", |m| box move |e| {
                if Instant::now() > start_cloned {
                    return vec![Hook::unregister(m)];
                }

                let mut wars = wars();
                match wars.get_mut(&w) {
                    None => return vec![Hook::unregister(m)],
                    Some(mut current_war) => {
                        if current_war.participants.contains(&string!(e.sender)) {
                            e.respond_highlight("You're already in this war.");
                        } else {
                            current_war.participants.insert(string!(e.sender));
                            current_war.register_msgs(&e);
                            e.respond_highlight(format!("You've been added to war #{}.", w2));
                        }
                    }
                }

                vec![]
            }), Hook::register("out", |m| box move |e| {
                if Instant::now() > end_cloned {
                    return vec![Hook::unregister(m)];
                }

                let mut wars = wars();
                match wars.get_mut(&w) {
                    None => return vec![Hook::unregister(m)],
                    Some(mut current_war) => {
                        if current_war.participants.contains(&string!(e.sender)) {
                            current_war.participants.remove(&string!(e.sender));
                            current_war.register_msgs(&e);
                            e.respond_highlight(format!("You've been removed from war #{}.", w2));
                        } else {
                            e.respond_highlight("You're not in this war.");
                        }
                    }
                }

                vec![]
            })]
        },
        Err(s) => { e.respond_highlight(s); }
    }

    vec![]
}

pub fn wars_init() {
    WARS.set(Mutex::new(HashMap::new()));
}
