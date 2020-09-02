#[cfg(not(target_arch = "wasm32"))]
use glfw::{Context as _, WindowEvent};
use legion::*;
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::blending::{Blending, Equation, Factor};
use luminance_front::context::GraphicsContext;
use luminance_front::pipeline::{PipelineState, TextureBinding};
use luminance_front::pixel::{NormRGBA8UI, NormUnsigned};
use luminance_front::render_state::RenderState;
use luminance_front::shader::{Program, Uniform};
use luminance_front::tess::{Mode, Tess};
use luminance_front::texture::{Dim2, GenMipmaps, MagFilter, Sampler, Texture};
#[cfg(not(target_arch = "wasm32"))]
use luminance_glfw::GlfwSurface;
#[cfg(target_arch = "wasm32")]
use luminance_web_sys::WebSysWebGL2Surface;
use luminance_windowing::{WindowDim, WindowOpt};
use nalgebra::{Matrix4, Vector3, Vector4};

use crate::components::{Player, Sprite, Transform};
use crate::resources::WorldBounds;
use crate::spritesheet::Spritesheet;
use rapier2d::dynamics::RigidBodyHandle;

const SPRITE_VS: &str = include_str!("texture-vs.glsl");
const SPRITE_FS: &str = include_str!("texture-fs.glsl");

const DEFAULT_VS: &str = include_str!("vs.glsl");
const DEFAULT_FS: &str = include_str!("fs.glsl");

const SPRITESHEET: &[u8] = include_bytes!("spritesheet.png");

#[cfg(target_arch = "wasm32")]
type RenderSurface = WebSysWebGL2Surface;
#[cfg(not(target_arch = "wasm32"))]
type RenderSurface = GlfwSurface;

const SPRITES_PER_HALF_SCREEN: f32 = 15.;

const COLLIDER_VERTICES: [ColliderVertex; 5] = [
    ColliderVertex {
        pos: VertexPosition::new([0., 0.]),
    },
    ColliderVertex {
        pos: VertexPosition::new([0., 1.]),
    },
    ColliderVertex {
        pos: VertexPosition::new([1., 1.]),
    },
    ColliderVertex {
        pos: VertexPosition::new([1., 0.]),
    },
    ColliderVertex {
        pos: VertexPosition::new([0., 0.]),
    },
];

const INSTANCES: [Instance; 9] = [
    Instance {
        offset: VertexInstancePosition::new([0., 0.]),
    },
    Instance {
        offset: VertexInstancePosition::new([-1., 0.]),
    },
    Instance {
        offset: VertexInstancePosition::new([-1., -1.]),
    },
    Instance {
        offset: VertexInstancePosition::new([0., -1.]),
    },
    Instance {
        offset: VertexInstancePosition::new([1., -1.]),
    },
    Instance {
        offset: VertexInstancePosition::new([1., 0.]),
    },
    Instance {
        offset: VertexInstancePosition::new([1., 1.]),
    },
    Instance {
        offset: VertexInstancePosition::new([0., 1.]),
    },
    Instance {
        offset: VertexInstancePosition::new([-1., 1.]),
    },
];

pub struct Renderer {
    surface: RenderSurface,
    default_shader: Program<Semantics, (), DefaultShaderInterface>,
    sprite_shader: Program<Semantics, (), SpriteShaderInterface>,
    projection: Matrix4<f32>,
    spritesheet: Spritesheet,
    tesses: Vec<Tess<Vertex, (), Instance>>,
}

impl Renderer {
    pub fn new() -> Self {
        let mut surface = create_surface();
        let img = read_image(SPRITESHEET).expect("Failed to load spritesheet");
        let tex = load_texture(&mut surface, img);
        let [w, h] = tex.size();
        let spritesheet = Spritesheet::new(tex, w, h, 32);
        let mut tesses = vec![];
        for i in 0..(w * h / 32) {
            let tess = surface
                .new_tess()
                .set_vertices(
                    spritesheet
                        .get_vertices(i)
                        .iter()
                        .map(|d| Vertex {
                            pos: VertexPosition::new(d.0),
                            tex_coords: TexturePosition::new(d.1),
                        })
                        .collect::<Vec<Vertex>>(),
                )
                .set_instances(&INSTANCES[..])
                .set_mode(Mode::TriangleFan)
                .build()
                .unwrap();
            tesses.push(tess);
        }

        let aspect_ratio = 960. / 540.;
        let projection = Matrix4::new_orthographic(
            -SPRITES_PER_HALF_SCREEN,
            SPRITES_PER_HALF_SCREEN,
            -SPRITES_PER_HALF_SCREEN / aspect_ratio,
            SPRITES_PER_HALF_SCREEN / aspect_ratio,
            -1.,
            1.,
        );
        let default_shader = surface
            .new_shader_program::<Semantics, (), DefaultShaderInterface>()
            .from_strings(DEFAULT_VS, None, None, DEFAULT_FS)
            .expect("Shader program creation")
            .ignore_warnings();
        let sprite_shader = surface
            .new_shader_program::<Semantics, (), SpriteShaderInterface>()
            .from_strings(SPRITE_VS, None, None, SPRITE_FS)
            .expect("Shader program creation")
            .ignore_warnings();
        Renderer {
            surface,
            default_shader,
            sprite_shader,
            projection,
            spritesheet,
            tesses,
        }
    }

    pub fn draw(&mut self, world: &mut World, resources: &Resources) {
        let back_buffer = self.surface.back_buffer().unwrap();
        let render_st = RenderState::default()
            .set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::SrcAlphaComplement,
            })
            .set_depth_test(None);

        let sprite_program = &mut self.sprite_shader;
        let default_program = &mut self.default_shader;
        let tex = &mut self.spritesheet.texture;
        let projection = self.projection;
        let tesses = &self.tesses;
        let collider_tess = self
            .surface
            .new_tess()
            .set_vertices(&COLLIDER_VERTICES[..])
            .set_instances(&INSTANCES[..])
            .set_mode(Mode::LineStrip)
            .build()
            .unwrap();

        self.surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |pipeline, mut shading_gate| {
                    let bound_tex = pipeline.bind_texture(tex)?;
                    let view: Matrix4<f32> = {
                        <(&Transform, &Player)>::query()
                            .iter(world)
                            .find_map(|(t, _)| {
                                t.isometry.translation.to_homogeneous().try_inverse()
                            })
                            .unwrap()
                    };

                    let bounds = resources.get::<WorldBounds>().unwrap();

                    shading_gate.shade(sprite_program, |mut iface, uni, mut render_gate| {
                        iface.set(&uni.image, bound_tex.binding());
                        iface.set(&uni.projection, projection.into());
                        iface.set(&uni.pc0, projection.column(0).into());
                        iface.set(&uni.pc1, projection.column(1).into());
                        iface.set(&uni.pc2, projection.column(2).into());
                        iface.set(&uni.pc3, projection.column(3).into());
                        iface.set(&uni.view, view.into());
                        iface.set(&uni.vc0, view.column(0).into());
                        iface.set(&uni.vc1, view.column(1).into());
                        iface.set(&uni.vc2, view.column(2).into());
                        iface.set(&uni.vc3, view.column(3).into());
                        iface.set(&uni.world_bounds, bounds.as_f32().into());

                        let mut sprite_query = <(&Sprite, &Transform)>::query();
                        for (sprite, transform) in sprite_query.iter(world) {
                            let mut model = transform.get_matrix();
                            model *= Matrix4::new_translation(&Vector3::new(-0.5, -0.5, 0.));
                            iface.set(&uni.model, model.into());
                            iface.set(&uni.mc0, model.column(0).into());
                            iface.set(&uni.mc1, model.column(1).into());
                            iface.set(&uni.mc2, model.column(2).into());
                            iface.set(&uni.mc3, model.column(3).into());
                            iface.set(&uni.sprite_color, sprite.color);
                            render_gate.render(&render_st, |mut tess_gate| {
                                tess_gate.render(&tesses[sprite.index])
                            })?
                        }

                        Ok(())
                    })?;
                    shading_gate.shade(default_program, |mut iface, uni, mut render_gate| {
                        iface.set(&uni.projection, projection.into());
                        iface.set(&uni.pc0, projection.column(0).into());
                        iface.set(&uni.pc1, projection.column(1).into());
                        iface.set(&uni.pc2, projection.column(2).into());
                        iface.set(&uni.pc3, projection.column(3).into());
                        iface.set(&uni.view, view.into());
                        iface.set(&uni.vc0, view.column(0).into());
                        iface.set(&uni.vc1, view.column(1).into());
                        iface.set(&uni.vc2, view.column(2).into());
                        iface.set(&uni.vc3, view.column(3).into());
                        let mut collider_query = <(&RigidBodyHandle, &Transform)>::query();
                        for (_, transform) in collider_query.iter(world) {
                            let mut model = transform.get_matrix();
                            model *= Matrix4::new_translation(&Vector3::new(-0.5, -0.5, 0.));
                            iface.set(&uni.model, model.into());
                            iface.set(&uni.mc0, model.column(0).into());
                            iface.set(&uni.mc1, model.column(1).into());
                            iface.set(&uni.mc2, model.column(2).into());
                            iface.set(&uni.mc3, model.column(3).into());
                            iface.set(&uni.v_color, [1.0, 0., 0.]);
                            render_gate.render(&render_st, |mut tess_gate| {
                                tess_gate.render(&collider_tess)
                            })?
                        }
                        Ok(())
                    })
                },
            )
            .assume()
            .into_result()
            .unwrap();

        swap_buffers(&mut self.surface);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn iter_events(&mut self) -> std::sync::mpsc::TryIter<(f64, WindowEvent)> {
        self.surface.window.glfw.poll_events();
        self.surface.events_rx.try_iter()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer::new()
    }
}

#[cfg(target_arch = "wasm32")]
fn swap_buffers(_: &mut RenderSurface) {}
#[cfg(not(target_arch = "wasm32"))]
fn swap_buffers(surface: &mut RenderSurface) {
    surface.window.swap_buffers();
}

fn load_texture(surface: &mut RenderSurface, img: image::RgbaImage) -> Texture<Dim2, NormRGBA8UI> {
    let (width, height) = img.dimensions();
    let texels = img.into_raw();

    let mut sampler = Sampler::default();
    sampler.mag_filter = MagFilter::Nearest;
    // create the luminance texture; the third argument is the number of mipmaps we want (leave it
    // to 0 for now) and the latest is the sampler to use when sampling the texels in the
    // shader
    let mut tex = surface
        .new_texture([width, height], 0, sampler)
        .expect("luminance texture creation");

    // the first argument disables mipmap generation (we don’t care so far)
    tex.upload_raw(GenMipmaps::Yes, &texels).unwrap();

    tex
}

fn read_image(buf: &[u8]) -> Option<image::RgbaImage> {
    image::load_from_memory(buf)
        .map(|img| img.flipv().to_rgba())
        .ok()
}

#[cfg(target_arch = "wasm32")]
pub fn create_surface() -> WebSysWebGL2Surface {
    let dim = WindowDim::Windowed {
        width: 960,
        height: 540,
    };
    WebSysWebGL2Surface::new("game", WindowOpt::default().set_dim(dim))
        .ok()
        .unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_surface() -> GlfwSurface {
    let dim = WindowDim::Windowed {
        width: 960,
        height: 540,
    };
    GlfwSurface::new_gl33("No Tilearino", WindowOpt::default().set_dim(dim))
        .ok()
        .unwrap()
}
#[derive(UniformInterface)]
struct DefaultShaderInterface {
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
    view: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    vc0: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    vc1: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    vc2: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    vc3: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    v_color: Uniform<[f32; 3]>,
}

#[derive(UniformInterface)]
struct SpriteShaderInterface {
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
    view: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    vc0: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    vc1: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    vc2: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    vc3: Uniform<[f32; 4]>,
    #[uniform(unbound)]
    image: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    #[uniform(unbound)]
    sprite_color: Uniform<[f32; 3]>,
    #[uniform(unbound)]
    world_bounds: Uniform<[f32; 2]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
    #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "tex_co", repr = "[f32; 2]", wrapper = "TexturePosition")]
    Color,
    #[sem(
        name = "position",
        repr = "[f32; 2]",
        wrapper = "VertexInstancePosition"
    )]
    InstancePosition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
    pos: VertexPosition,
    tex_coords: TexturePosition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics", instanced = "true")]
pub struct Instance {
    pub offset: VertexInstancePosition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct ColliderVertex {
    pos: VertexPosition,
}
