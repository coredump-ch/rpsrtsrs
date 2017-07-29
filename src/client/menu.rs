use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;

use piston::input::{RenderArgs};

use colors::{BLACK, YELLOW, ORANGE};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Entries {
    Start,
    Exit,
}

impl Entries {
    pub fn next(&mut self) {
        *self = match *self {
            Entries::Start => Entries::Exit,
            Entries::Exit => Entries::Start,
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Menu {
    selected_entry: Entries,
}

impl Menu {
    pub fn new() -> Menu {
        Menu { selected_entry: Entries::Start }
    }

    pub fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics, cache: &mut GlyphCache) {
        use graphics::{Text, clear, Transformed};
        let text = Text::new_color(YELLOW, 64);
        let text_selected = Text::new_color(ORANGE, 64);
        gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            let mut transform = c.transform;

            for entry in &[Entries::Start, Entries::Exit] {
                transform = transform.trans(0.0, 100.0);
                if *entry == self.selected_entry {
                    text_selected.draw(&format!("{:?}", entry), cache, &c.draw_state, transform, gl);
                } else {
                    text.draw(&format!("{:?}", entry), cache, &c.draw_state, transform, gl);
                }
            }
        });
    }

    pub fn get_selected_entry(&self) -> Entries {
        self.selected_entry
    }

    pub fn previous(&mut self) {
        self.selected_entry.next();
    }

    pub fn next(&mut self) {
        self.selected_entry.next();
    }

}
