# rocket_casbin_demo

Rocket rely on nightly.

`rustup default nightly`

(optional)

Init casbin policy database. I've created this in db folder.

`cargo run --bin init_db`

Run with file adapter.

`cargo run --bin file_adapter`

Or, run with diesel adapter

`cargo run --bin orm_adapter`.

Test

get http://localhost:8000/user?name=alice