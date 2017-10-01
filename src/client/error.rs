use opengl_graphics::GlGraphics;
use opengl_graphics::GlyphCache;

use piston::input::{RenderArgs};

use colors::{BLACK, YELLOW};

#[derive(Clone, Debug)]
pub struct Message {
    message: String,
}

impl Message {
    pub fn new(message: String) -> Message {
        Message { message: message }
    }

    pub fn render(&self, args: &RenderArgs, gl: &mut GlGraphics, cache: &mut GlyphCache) {
        use graphics::{Text, clear, Transformed};
        let text = Text::new_color(YELLOW, 64);
        gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            let transform = c.transform.trans(0.0, 100.0);
            text.draw(&self.message, cache, &c.draw_state, transform, gl);
        });
    }
}

