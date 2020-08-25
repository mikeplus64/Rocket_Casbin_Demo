#![feature(proc_macro_hygiene, decl_macro)]

use casbin::prelude::*;
use rocket::{
    get,
    http::{Method, RawStr},
    request::{self, FromRequest, Request},
    routes,
    Outcome::{Forward, Success},
};
use rocket_contrib::database;
use diesel_adapter::DieselAdapter;

#[database("user_db")]
#[derive(Debug)]
pub struct DbConn(rocket_contrib::databases::diesel::SqliteConnection);

pub struct User {
    name: String,
}

// get http://localhost:8000/user?name=alice
// if 
#[get("/user?<name>")]
pub fn user(name: String, _user: User) -> String {
    format!("{} is valid", name)
}

// 
#[get("/user?<name>", rank = 2)]
pub fn not_enforced(name: String) -> String {
    format!("{} is invalid", name)
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let enforcer = request.guard::<rocket::State<Enforcer>>()?;
        let name = {
            let n: &RawStr = request
                .get_query_value("name")
                .and_then(|r| r.ok())
                .unwrap_or("".into());
            n.to_string()
        };

        let path = request.uri().path().to_owned();
        let method = match request.method() {
            Method::Get => "GET".to_owned(),
            _ => "Invalid".to_owned(),
        };

        let val = vec![&name, &path, &method];
        if let Ok(true) = enforcer.enforce(&val) {
            Success(User { name })
        } else {
            Forward(())
        }
    }
}

fn await_future<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}


fn main() {
    let m = await_future(DefaultModel::from_file("casbin_conf/model.conf")).unwrap();
    let a = DieselAdapter::new().unwrap();
    let mut e = await_future(Enforcer::new(m, a)).unwrap();
    e.enable_auto_save(true);
    
    rocket::ignite()
        .manage(e)
        .mount("/", routes![user, not_enforced])
        .launch();
}
