use luminance_front::pixel::NormRGBA8UI;
use luminance_front::texture::{Dim2, Texture};

type VertexQuad = [([f32; 2], [f32; 2]); 4];

pub struct Spritesheet {
    rows: u32,
    columns: u32,
    pub texture: Texture<Dim2, NormRGBA8UI>,
}

impl Spritesheet {
    pub fn new(tex: Texture<Dim2, NormRGBA8UI>, width: u32, height: u32, size: u32) -> Self {
        Spritesheet {
            rows: height / size,
            columns: width / size,
            texture: tex,
        }
    }

    pub fn get_vertices(&self, sprite_index: u32) -> VertexQuad {
        compute_coords(sprite_index, self.columns, self.rows)
    }
}

fn compute_coords(idx: u32, cols: u32, rows: u32) -> VertexQuad {
    let row = idx / cols;
    let col = idx % cols;
    let tl = (col as f32 / cols as f32, row as f32 / rows as f32);
    let br = (
        (col + 1) as f32 / cols as f32,
        (row + 1) as f32 / rows as f32,
    );
    [
        ([0., 0.], [tl.0, tl.1]),
        ([1., 0.], [br.0, tl.1]),
        ([1., 1.], [br.0, br.1]),
        ([0., 1.], [tl.0, br.1]),
    ]
}
