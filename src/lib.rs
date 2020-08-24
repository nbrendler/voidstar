#[macro_use]
mod utils;

use cgmath::{ortho, Matrix4, SquareMatrix, Vector3};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::blending::{Blending, Equation, Factor};
use luminance_front::context::GraphicsContext;
use luminance_front::pipeline::{PipelineState, TextureBinding};
use luminance_front::pixel::{NormRGB8UI, NormUnsigned};
use luminance_front::render_state::RenderState;
use luminance_front::shader::{Program, Uniform};
use luminance_front::tess::Mode;
use luminance_front::texture::{Dim2, GenMipmaps, Sampler, Texture};
use luminance_web_sys::WebSysWebGL2Surface;
use luminance_windowing::WindowOpt;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}

#[derive(UniformInterface)]
struct ShaderInterface {
    #[uniform(unbound)]
    model: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    mc0: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    mc1: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    mc2: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    mc3: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    pc0: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    pc1: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    pc2: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    pc3: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    image: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    #[uniform(unbound)]
    sprite_color: Uniform<[f32; 3]>,
}

const VS: &str = include_str!("texture-vs.glsl");
const FS: &str = include_str!("texture-fs.glsl");

const SPRITESHEET: &[u8] = include_bytes!("spritesheet.png");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
    #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "tex_coords", repr = "[f32; 2]", wrapper = "TexturePosition")]
    Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
    pos: VertexPosition,
    tex_coords: TexturePosition,
}

const QUAD_VERTICES: [Vertex; 4] = [
    Vertex::new(
        VertexPosition::new([0., 0.]),
        TexturePosition::new([0., 0.]),
    ),
    Vertex::new(
        VertexPosition::new([1., 0.]),
        TexturePosition::new([1., 0.]),
    ),
    Vertex::new(
        VertexPosition::new([1., 1.]),
        TexturePosition::new([1., 1.]),
    ),
    Vertex::new(
        VertexPosition::new([0., 1.]),
        TexturePosition::new([0., 1.]),
    ),
];

#[wasm_bindgen]
pub struct Game {
    model: Matrix4<f32>,
    projection: Matrix4<f32>,
    texture: Texture<Dim2, NormRGB8UI>,
    surface: WebSysWebGL2Surface,
    shader_program: Program<Semantics, (), ShaderInterface>,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        let mut surface = WebSysWebGL2Surface::new("game", WindowOpt::default())
            .ok()
            .unwrap();
        let program = surface
            .new_shader_program::<Semantics, (), ShaderInterface>()
            .from_strings(VS, None, None, FS)
            .expect("Shader program creation")
            .ignore_warnings();
        let img = read_image(SPRITESHEET).unwrap();
        let tex = load_from_disk(&mut surface, img);
        let model = Matrix4::identity();
        let projection = ortho(0., 960., 0., 540., -1., 1.);
        Game {
            surface,
            shader_program: program,
            texture: tex,
            model,
            projection,
        }
    }

    pub fn tick(&mut self) {
        let back_buffer = self.surface.back_buffer().unwrap();
        let color = [0.141, 0.141, 0.141, 1.];
        let render_st = &RenderState::default().set_blending(Blending {
            equation: Equation::Additive,
            src: Factor::SrcAlpha,
            dst: Factor::DstAlpha,
        });

        let tess = self
            .surface
            .new_tess()
            .set_vertices(&QUAD_VERTICES[..])
            .set_mode(Mode::TriangleFan)
            .build()
            .unwrap();

        let program = &mut self.shader_program;
        let tex = &mut self.texture;
        let mut model = self.model;
        let projection = self.projection;

        let pos = Matrix4::from_translation(Vector3::new(480., 270., 0.));
        let scale = Matrix4::from_scale(32.);
        model = model * pos;
        model = model * scale;

        self.surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color),
                |pipeline, mut shading_gate| {
                    let bound_tex = pipeline.bind_texture(tex)?;
                    shading_gate.shade(program, |mut iface, uni, mut render_gate| {
                        iface.set(&uni.image, bound_tex.binding());
                        iface.set(&uni.sprite_color, [37. / 255., 113. / 255., 121. / 255.]);
                        iface.set(&uni.model, model.into());
                        iface.set(&uni.mc0, model.x.into());
                        iface.set(&uni.mc1, model.y.into());
                        iface.set(&uni.mc2, model.z.into());
                        iface.set(&uni.mc3, model.w.into());
                        iface.set(&uni.projection, projection.into());
                        iface.set(&uni.pc0, projection.x.into());
                        iface.set(&uni.pc1, projection.y.into());
                        iface.set(&uni.pc2, projection.z.into());
                        iface.set(&uni.pc3, projection.w.into());
                        render_gate.render(render_st, |mut tess_gate| tess_gate.render(&tess))
                    })
                },
            )
            .assume()
            .into_result()
            .unwrap();
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

// read the texture into memory as a whole bloc (i.e. no streaming)
fn read_image(buffer: &[u8]) -> Option<image::RgbImage> {
    image::load_from_memory(buffer)
        .map(|img| img.flipv().to_rgb())
        .ok()
}

fn load_from_disk(
    surface: &mut WebSysWebGL2Surface,
    img: image::RgbImage,
) -> Texture<Dim2, NormRGB8UI> {
    let (width, height) = img.dimensions();
    let texels = img.into_raw();

    // create the luminance texture; the third argument is the number of mipmaps we want (leave it
    // to 0 for now) and the latest is the sampler to use when sampling the texels in the
    // shader (we’ll just use the default one)
    let mut tex = Texture::new(surface, [width, height], 0, Sampler::default())
        .expect("luminance texture creation");

    // the first argument disables mipmap generation (we don’t care so far)
    tex.upload_raw(GenMipmaps::No, &texels).unwrap();

    tex
}
