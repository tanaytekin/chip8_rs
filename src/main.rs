mod chip8;
mod app;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

fn main() {
    let mut app = app::App::new();
    app.run();
}
