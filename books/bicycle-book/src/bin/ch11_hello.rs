use actix_web::{server, App, Error, HttpRequest, Path, Responder, State};
use serde_derive::*;

fn hello<T>(req: &HttpRequest<T>) -> impl Responder {
    let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", to)
}

#[derive(Deserialize)]
struct HelloPath {
    name: String,
}

fn hello_name(to: Path<HelloPath>) -> impl Responder {
    format!("Hello {}!", &to.name)
}

struct MyApp {
    server_name: String,
}

fn hello_with_state(app: State<MyApp>) -> Result<String, Error> {
    Ok(format!("Hello from {}!", &app.server_name))
}

fn main() {
    server::new(|| {
        App::with_state(MyApp {
            server_name: "server with state".into(),
        })
        .resource("/info", |r| r.with(hello_with_state))
        .resource("/", |r| r.f(hello))
        .resource("/{name}", |r| r.with(hello_name))
    })
    .bind("localhost:3000")
    .expect("Can not bind to port 3000")
    .run();
}
