# rocket_casbin_demo

this is a template for rocket proj with casbin.

All you need to do is that rewrite the variables in [CasbinFairing](https://github.com/simoin/Rocket_Casbin_Demo/blob/1517ee67338049bdfa38d2240bfdabcd786ec25c/src/main.rs#L42)
```rust
fn on_request(&self, request: &mut Request<'r>, _data: &Data) {
    let sub = String::from("alice");
    let domain = None;
    let res = request.uri().path().to_owned();
    let action = request.method().as_str().to_owned();
    let cloned_enforce = self.enforcer.clone();

    ...
}
```