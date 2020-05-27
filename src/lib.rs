mod drawer;
mod event;
mod pty_driver;
mod term;

#[macro_use]
extern crate gdnative;

fn init(handle: gdnative::init::InitHandle) {
    handle.add_tool_class::<drawer::Drawer>();
    handle.add_tool_class::<term::Term>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
