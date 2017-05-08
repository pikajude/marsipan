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

#[derive(Debug)]
struct War {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    start_msg: Instant,
    end_msg: Instant,
    participants: HashSet<String>,
    starter: String,
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
            match wars().get(&h) {
                Some(w) => if w.starter == string!(e.sender) {
                    let war = wars().remove(&h).unwrap();
                    e.cancel(war.start_msg);
                    e.cancel(war.end_msg);
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

            let start = e.respond_in(format!("{}: <b>START WRITING!</b>", string!(e.sender)), until(start_instant).unwrap());
            let end = e.respond_in(format!("{}: <b>STOP WRITING!</b>", string!(e.sender)), until(end_instant).unwrap());

            e.respond_highlight(format!("Scheduled war with ID #{}.", w));

            let start_cloned = start.clone();
            let w2 = w.clone();

            wars().insert(w, War {
                start_time: start_instant,
                end_time: end_instant,
                start_msg: start,
                end_msg: end,
                participants: {
                    let mut h = HashSet::new();
                    h.insert(string!(e.sender));
                    h
                },
                starter: string!(e.sender),
            });

            return vec![Hook::register("in", |m| box move |e| {
                if Instant::now() > start_cloned {
                    return vec![Hook::unregister(m)];
                }

                let mut wars = wars();
                match wars.get_mut(&w) {
                    None => return vec![Hook::unregister(m)],
                    Some(ref mut current_war) => {
                        if current_war.participants.contains(&string!(e.sender)) {
                            e.respond_highlight("You're already in this war.");
                        } else {
                            current_war.participants.insert(string!(e.sender));
                            e.respond_highlight(format!("You've been added to war #{}.", w2));
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
