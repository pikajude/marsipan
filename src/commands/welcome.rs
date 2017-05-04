use commands::prelude::*;
use diesel;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

mod models {
    use super::schema::welcomes;

    #[derive(Queryable,Debug)]
    pub struct Welcome {
        id: i32,
        user: String,
        pub body: String,
    }

    #[derive(Insertable,Debug)]
    #[table_name="welcomes"]
    pub struct NewWelcome {
        pub user: String,
        pub body: String,
    }
}

mod schema {
    infer_schema!("dotenv:DATABASE_URL");
}

macro_rules! string {
    ($x:expr) => { $x.iter().map(|&c|c as char).collect::<String>() }
}

macro_rules! welcome_for {
    ($x:expr) => {
        {
            use self::models::*;
            use self::schema::welcomes::dsl::*;

            welcomes.filter(user.eq(string!($x)))
        }
    }
}

pub fn say_welcome(e: Event) -> Hooks {
    use self::models::Welcome;

    if let Some(w) = e.load(welcome_for!(e.sender)).into_iter().next() as Option<Welcome> {
        e.respond(w.body);
    }

    vec![]
}

pub fn welcome(e: Event) -> Hooks {
    use self::models::{NewWelcome,Welcome};
    use self::schema::welcomes::dsl::*;

    match word(&e.content()) {
        ("get", _) => {
            match e.load(welcome_for!(e.sender)).into_iter().next() as Option<Welcome> {
                Some(w) => e.respond_highlight(format!("Your welcome is '{}'", w.body)),
                None => e.respond_highlight("You don't have a welcome."),
            };
        },
        ("set", "") => {
            e.respond_highlight("Usage: !welcome set <b>thing</b><br>\
                                If you want no welcome, use !welcome clear.");
        },
        ("set", x) => {
            e.execute(diesel::insert_or_replace(&NewWelcome {
                user: string!(e.sender),
                body: x.to_string()
            }).into(welcomes::table()));
            e.respond_highlight("Your welcome has been set.");
        },
        ("clear", _) => {
            e.execute(diesel::delete(welcomes.filter(user.eq(string!(e.sender)))));
            e.respond_highlight("Your welcome has been forgotten.");
        }
        (_, _) => {
            e.respond_highlight("Usage: !welcome { get | set <i>welcome</i> | clear }");
        },
    };

    vec![]
}
