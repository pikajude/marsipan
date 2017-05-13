use commands::prelude::*;
use diesel;
use diesel::associations::HasTable;

mod models {
    use ::db::welcomes;

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

    impl Welcome {
        pub fn belongs_to(u: &[u8]) -> ::diesel::helper_types::FindBy<
                ::db::welcomes::dsl::welcomes,
                ::db::welcomes::dsl::user,
                String> {
            use ::db::welcomes::dsl::*;

            use diesel::FilterDsl;
            use diesel::ExpressionMethods;

            welcomes.filter(user.eq(string!(u)))
        }
    }
}

pub fn say_welcome(e: &Event) -> Hooks {
    use self::models::Welcome;

    if let Some(w) = e.load(Welcome::belongs_to(&e.sender))
                      .into_iter().next() as Option<Welcome> {
        e.respond(w.body);
    }

    vec![]
}

pub fn welcome(e: &Event) -> Hooks {
    use self::models::{NewWelcome,Welcome};
    use ::db::welcomes::dsl::*;

    match word(&e.content()) {
        ("get", _) => {
            match e.load(Welcome::belongs_to(&e.sender)).into_iter().next() as Option<Welcome> {
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
            let affected = e.execute(diesel::delete(Welcome::belongs_to(&e.sender)));
            if affected > 0 {
                e.respond_highlight("Your welcome has been forgotten.");
            } else {
                e.respond_highlight("You didn't have a welcome.");
            }
        },
        (_, _) => {
            e.respond_highlight("Usage: !welcome { get | set <i>welcome</i> | clear }");
        },
    };

    vec![]
}
