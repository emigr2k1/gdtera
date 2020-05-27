use aterm::{
    clipboard::Clipboard,
    config::Config,
    event_loop::Msg,
    sync::FairMutex,
    term::{RenderableCell, SizeInfo},
    tty, Term,
};
use std::sync::{mpsc, Arc, Mutex};

use super::event::Event;

pub(crate) struct PtyDriver {
    term_tx: mio_extras::channel::Sender<Msg>,
    term: Arc<FairMutex<Term<TermListener>>>,
    config: Config<()>,
}

impl PtyDriver {
    pub(crate) fn new() -> (Self, mpsc::Receiver<Event>) {
        let mut config = Config::default();

        config.shell = Some(aterm::config::Shell {
            program: "/usr/local/bin/fish".into(),
            args: vec![],
        });

        let (godot_tx, godot_rx) = mpsc::channel::<Event>();
        let listener = TermListener(Arc::new(Mutex::new(godot_tx)));

        aterm::tty::setup_env(&config);

        #[cfg(not(any(target_os = "macos", windows)))]
        let clipboard = Clipboard::new(display.window.wayland_display());
        #[cfg(any(target_os = "macos", windows))]
        let clipboard = Clipboard::new();

        let size_info = &SizeInfo {
            width: 180.,
            height: 50.,
            cell_width: 1.,
            cell_height: 1.,
            padding_x: 0.,
            padding_y: 0.,
            dpr: 96.,
        };
        let mut terminal = Term::new(&config, &size_info, clipboard, listener.clone());
        terminal.is_focused = true;
        terminal.dirty = true;
        let terminal = Arc::new(FairMutex::new(terminal));

        #[cfg(not(any(target_os = "macos", windows)))]
        let pty = tty::new(&config, &size_info, display.window.x11_window_id());
        #[cfg(any(target_os = "macos", windows))]
        let pty = tty::new(&config, &size_info, None);

        let event_loop =
            aterm::event_loop::EventLoop::new(terminal.clone(), listener.clone(), pty, &config);
        let event_tx = event_loop.channel();

        event_loop.spawn();

        (
            PtyDriver {
                term_tx: event_tx,
                term: terminal,
                config,
            },
            godot_rx,
        )
    }

    pub fn on_input(&mut self, event: gdnative::InputEvent) {
        godot_print!("-> PtyDriver::on_input");
        if !event.is_pressed() {
            return;
        }

        let key_event = event.cast::<gdnative::InputEventKey>();
        let e = if let Some(e) = key_event { e } else { return };

        use gdnative::GlobalConstants as gc;

        let code = e.get_scancode();

        let bytes = if code == gc::KEY_BACKSPACE {
            vec![8u8]
        } else if code == gc::KEY_ENTER {
            vec![10u8]
        } else if code == gc::KEY_SPACE {
            vec![32u8]
        } else if code == gc::KEY_ESCAPE {
            vec![27u8]
        } else if code == gc::KEY_TAB {
            vec![9u8]
        } else {
            let utf8_i64 = e.get_unicode();
            if utf8_i64 == 0 {
                return;
            }
            let mut bytes = Vec::with_capacity(8);
            for i in 0..(utf8_i64 as f64 / 0xFF as f64).ceil() as i64 {
                bytes.push(((utf8_i64 >> (i * 8)) & 0xFF) as u8);
            }
            bytes
        };

        self.term_tx.send(Msg::Input(bytes.into())).unwrap();
        godot_print!("<- PtyDriver::on_input");
    }

    pub(crate) fn renderable_cells(&mut self) -> Vec<RenderableCell> {
        godot_print!("-> PtyDriver::renderable_cells");
        let term = self.term.lock();
        let cells = term.renderable_cells(&self.config).collect::<Vec<_>>();
        godot_print!("<- PtyDriver::renderable_cells");
        cells
    }
}

#[derive(Clone)]
struct TermListener(Arc<Mutex<mpsc::Sender<Event>>>);

impl aterm::event::EventListener for TermListener {
    fn send_event(&self, event: aterm::event::Event) {
        match event {
            aterm::event::Event::Wakeup => self.0.lock().unwrap().send(Event::WakeUp).unwrap(),
            _ => {}
        }
    }
}
