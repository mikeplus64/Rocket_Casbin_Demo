#![feature(proc_macro_hygiene, decl_macro)]
#![feature(in_band_lifetimes)]

use std::sync::RwLock;
use std::sync::Arc;
use casbin::prelude::*;
use rocket::{
    get,
    http:: Status,
    fairing::{Info, Kind, Fairing},
    request::{self, FromRequest, Request},
    routes,
    Data,
};

pub struct CasbinFairing {
    enforcer: Arc<RwLock<CachedEnforcer>>,
}

pub struct CasbinGuard(Option<Status>);

impl<'a, 'r> FromRequest<'a, 'r> for CasbinGuard {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<CasbinGuard, ()> {
        match *request.local_cache(|| CasbinGuard(Status::from_code(0))) {
            CasbinGuard(Some(Status::Ok)) => request::Outcome::Success(CasbinGuard(Some(Status::Ok))),
            CasbinGuard(Some(err_status)) => request::Outcome::Failure((err_status, ())),
            _ => request::Outcome::Failure((Status::BadGateway, ())),
        }
    }
}

impl Fairing for CasbinFairing {
    fn info(&self) -> Info {
        Info {
            name: "Casbin Fairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request<'r>, _data: &Data) {
        let sub = String::from("alice");
        let domain = None;
        let res = request.uri().path().to_owned();
        let action = request.method().as_str().to_owned();
        let cloned_enforce = self.enforcer.clone();

        let mut lock = cloned_enforce.write().unwrap();
        if let Some(domain) = &domain {
            match lock.enforce_mut(&[&sub, domain, &res, &action]) {
                Ok(true) => {
                    request.local_cache(|| CasbinGuard(Some(Status::Ok)));
                }
                Ok(false) => {
                    request.local_cache(|| CasbinGuard(Some(Status::Forbidden)));
                }
                Err(_) => {
                    request.local_cache(|| CasbinGuard(Some(Status::BadGateway)));
                }
            }
        } else {
            match lock.enforce_mut(&[&sub, &res, &action]) {
                Ok(true) => {
                    request.local_cache(|| CasbinGuard(Some(Status::Ok)));
                }
                Ok(false) => {
                    request.local_cache(|| CasbinGuard(Some(Status::Forbidden)));
                }
                Err(_) => {
                    request.local_cache(|| CasbinGuard(Some(Status::BadGateway)));
                }
            }
        }
    }
}

#[get("/pen")]
pub fn pen(_g: CasbinGuard) -> &'static str {
    "pen"
}

#[get("/book/1")]
pub fn book( _g: CasbinGuard) -> &'static str {
    "book"
}


fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let casbin_fairing = match rt.block_on( CachedEnforcer::new("casbin_conf/model.conf", "casbin_conf/policy.csv") )
    {
        Ok(e) => CasbinFairing{ enforcer: Arc::new(RwLock::new(e))},
        Err(_) => panic!(""),
    };
    rocket::ignite()
        .attach(casbin_fairing)
        .mount("/", routes![pen, book])
        .launch();
}

