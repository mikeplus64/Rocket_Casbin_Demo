use casbin::prelude::*;
use diesel_adapter::DieselAdapter;

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
    
    // add policies
    await_future(
        e.add_named_policies("p", vec![
            vec!["bob".to_owned(), "/login".to_owned(), "GET".to_owned()],
            vec!["admin".to_owned(), "/user".to_owned(), "GET".to_owned()],
        ])).unwrap();
    
    // add group
    await_future(
        e.add_named_grouping_policies("g", vec![
            vec!["alice".to_owned(), "admin".to_owned()],
        ])).unwrap();

    let res = e.enforce(&["alice", "/user", "GET"]);
    println!("{:#?}", res);
}
