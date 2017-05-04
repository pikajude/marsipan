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

    #[derive(Insertable)]
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

pub fn say_welcome(e: Event) -> Hooks {
    use self::models::*;
    use self::schema::welcomes::dsl::*;

    if let Some(w) = e.load(welcomes.filter(user.eq(string!(e.sender))))
                      .into_iter().next() as Option<Welcome> {
        e.respond(w.body);
    }

    vec![]
}

pub fn welcome(e: Event) -> Hooks {
    use self::models::*;
    use self::schema::welcomes::dsl::*;

    e.respond(format!("{:?}", e.sender));

    match word(&e.content()) {
        ("get", _) =>
            match e.load(welcomes.filter(user.eq(string!(e.sender)))).into_iter().next() as Option<Welcome> {
                Some(w) => e.respond_highlight(format!("Your welcome is '{}'", w.body)),
                None => e.respond_highlight("You don't have a welcome."),
            },
        (_, _) => e.respond_highlight("Usage: !welcome { get | set <i>welcome</i> | clear }"),
    };

    vec![]
}
