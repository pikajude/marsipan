use commands::prelude::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

mod models {
    #[derive(Queryable,Debug)]
    pub struct Welcome {
        id: i32,
        user: String,
        pub body: String,
    }
}

mod schema {
    infer_schema!("dotenv:DATABASE_URL");
}

macro_rules! string {
    ($x:expr) => { $x.iter().map(|&c|c as char).collect::<String>() }
}

pub fn welcome(e: Event) -> Hooks {
    use self::models::*;
    use self::schema::welcomes::dsl::*;

    let result: Option<Welcome> = e.load(welcomes.filter(user.eq(string!(e.sender)))).into_iter().next();

    if let Some(w) = result {
        e.respond(w.body);
    }

    vec![]
}
