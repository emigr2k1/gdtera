use gdnative::{methods, Image, ImageTexture, NativeClass, Texture};

use aterm::term::RenderableCell;
use raqote::{DrawOptions, DrawTarget, SolidSource, Source};

type Owner = gdnative::CenterContainer;

#[derive(NativeClass)]
#[inherit(Owner)]
pub(crate) struct Drawer {
    pub dirty: bool,
    pub cells: Vec<RenderableCell>,
    last_cells: Vec<RenderableCell>,
    image: Image,
    texture: Option<Texture>,
    last_size: (f32, f32),
}

#[methods]
impl Drawer {
    fn _init(_: Owner) -> Self {
        Self {
            dirty: false,
            cells: Vec::new(),
            last_cells: Vec::new(),
            image: Image::new(),
            texture: None,
            last_size: (0.0, 0.0),
        }
    }

    #[export]
    fn _ready(&mut self, owner: Owner) {
        self.last_size = (500.0, 500.0);

        godot_print!("Ready");

        self.image.create(self.last_size.0 as i64, self.last_size.1 as i64, false, 5);
    }

    #[export]
    unsafe fn _draw(&mut self, mut owner: Owner) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        godot_print!("-> _draw");

        let vp_size = owner.get_rect().size;
        if vp_size.width != self.last_size.0 || vp_size.height != self.last_size.1 {
            self.last_size = (vp_size.width, vp_size.height);
            self.image.resize(self.last_size.0 as i64, self.last_size.1 as i64, 1);
        }

        let mut dt = DrawTarget::new(vp_size.width as i32, vp_size.height as i32);

        let cell_width = 5.0;
        let cell_height = 8.0;

        let draw_opts = &DrawOptions::new();
        godot_print!("{:?}", self.cells);
        godot_print!("{:?}", self.last_cells);
        for (i, cell) in self.cells.iter().enumerate() {
            if let Some(last_cell) = self.last_cells.get(i) {
                if last_cell.bg == cell.bg && last_cell.bg_alpha == cell.bg_alpha {
                    continue;
                }
            }
            let _inner = cell.inner;

            let rect = (
                cell.column.0 as f32 * cell_width,
                cell.line.0 as f32 * cell_height,
                cell_width,
                cell_height,
            );

            let bg = Source::Solid(SolidSource {
                r: cell.bg.r,
                g: cell.bg.g,
                b: cell.bg.b,
                a: (cell.bg_alpha * 255.0) as u8,
            });

            dt.fill_rect(rect.0, rect.1, rect.2, rect.3, &bg, &draw_opts);
        }

        let data = dt.get_data();
        //img.lock();
        //img.resize((vp_size.width * cell_width) as i64, (vp_size.height * cell_height) as i64, 1);
        //img.unlock();

        //let mut image_data = ByteArray::new();
        //image_data.resize((vp_size.width * vp_size.height) as i32 * 4);

        self.image.lock();
        for (i, n) in data.iter().enumerate() {
            let i = i as i64;
            let b = (*n & 0xFF) as u8;
            let g = ((*n >> 8) & 0xFF) as u8;
            let r = ((*n >> 16) & 0xFF) as u8;
            let a = ((*n >> 24) & 0xFF) as u8;
            //image_data.set(i*4, r);
            //image_data.set(i*4+1, g);
            //image_data.set(i*4+2, b);
            //image_data.set(i*4+3, a);


            self.image.set_pixel(
                i % vp_size.width as i64,
                i / vp_size.width as i64,
                gdnative::Color::rgba(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ),
            );
        }
        self.image.unlock();

        //img.create_from_data(
        //    vp_size.width as i64,
        //    vp_size.height as i64,
        //    false,
        //    5,
        //    image_data,
        //);

        let mut img_text = ImageTexture::new();
        img_text.create_from_image(Some(self.image.clone()), 7);

        self.texture = Some(img_text.to_texture());

        owner.draw_texture(
            Some(self.texture.clone().unwrap()),
            (0.0, 0.0).into(),
            gdnative::Color::rgb(1.0, 1.0, 1.0),
            None,
        );
        self.last_cells = std::mem::take(&mut self.cells);
        godot_print!("<- _draw");
    }
}
