use gdnative::{
    methods, CanvasItem, EditorPlugin, FromVariant, GodotString, Instance, NativeClass, Node,
    PackedScene, ToVariant,
};

use super::{drawer::Drawer, event::Event, pty_driver::PtyDriver};

type Owner = EditorPlugin;

#[derive(NativeClass)]
#[inherit(Owner)]
pub(crate) struct Term {
    panel: Option<PackedScene>,
    panel_instance: Option<CanvasItem>,
    drawer: Option<Instance<Drawer>>,
    pty: PtyDriver,
    pty_rx: std::sync::mpsc::Receiver<Event>,
}

#[methods]
impl Term {
    fn _init(_owner: Owner) -> Self {
        let panel = gdnative::ResourceLoader::godot_singleton()
            .load(
                "res://addons/gdterm/main_panel.tscn".into(),
                "PackedScene".into(),
                false,
            )
            .unwrap()
            .cast::<PackedScene>()
            .unwrap();

        let (pty, pty_rx) = PtyDriver::new();

        Self {
            panel: Some(panel),
            panel_instance: None,
            drawer: None,
            pty,
            pty_rx,
        }
    }

    #[export]
    unsafe fn _process(&mut self, _owner: Owner, _delta: f32) {
        if self.pty_rx.try_recv().is_err() {
            return;
        }

        godot_print!("-> Term::_process");
        let cells = self.pty.renderable_cells();
        if let Some(drawer) = self.drawer.as_mut() {
            drawer
                .map_mut_aliased(|this, mut _owner| {
                    this.cells = cells;
                    this.dirty = true;
                    godot_print!("-> Drawer::update");
                    _owner.update();
                    godot_print!("<- Drawer::update");
                })
                .unwrap();
        }
        godot_print!("<- Term::_process");
    }

    #[export]
    unsafe fn _input(&mut self, owner: Owner, event: gdnative::InputEvent) {
        if !event.is_pressed() {
            return;
        }
        if event.cast::<gdnative::InputEventKey>().is_none() {
            return;
        }
        godot_print!("-> _input");
        self.pty.on_input(event);
        owner.get_tree().as_mut().unwrap().set_input_as_handled();
        godot_print!("<- _input");
    }

    #[export]
    unsafe fn _enter_tree(&mut self, mut owner: Owner) {
        self.panel_instance = Some(
            self.panel
                .as_ref()
                .unwrap()
                .instance(0)
                .unwrap()
                .cast::<CanvasItem>()
                .unwrap(),
        );

        owner
            .get_editor_interface()
            .unwrap()
            .get_editor_viewport()
            .unwrap()
            .cast::<Node>()
            .unwrap()
            .add_child(self.panel_instance.unwrap().cast::<Node>(), true);

        let variant = self.panel_instance.as_ref().unwrap().to_variant();
        self.drawer = Some(Instance::<Drawer>::from_variant(&variant).unwrap());

        self.make_visible(owner, false);
    }

    #[export]
    unsafe fn _exit_tree(&mut self, _: Owner) {
        if let Some(inst) = &mut self.panel_instance {
            inst.queue_free();
        }
    }

    #[export]
    fn has_main_screen(&self, _: Owner) -> bool {
        true
    }

    #[export]
    unsafe fn make_visible(&mut self, _: Owner, visible: bool) {
        if let Some(inst) = &mut self.panel_instance {
            inst.set_visible(visible);
        }
    }

    #[export]
    unsafe fn get_plugin_name(&self, _: Owner) -> GodotString {
        "GDTerm".into()
    }
}
