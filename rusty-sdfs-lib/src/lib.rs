#![allow(dead_code)]

mod animation;
mod canvas;
mod color;
mod grid;
mod noise;
mod ray_marcher;
mod render;
mod scene;
mod sdf;
mod streamline;
mod vector;

pub use animation::Animation;

pub use canvas::{Canvas, PixelPropertyCanvas, SkiaCanvas};

pub use color::LinearGradient;

pub use noise::{noise_2d, noisy_waves_heightmap, smoothstep};

pub use ray_marcher::RayMarcher;

pub use render::{render_flow_field_streamlines, DomainRegion, render_heightmap_streamlines, render_hatch_lines, render_edges};

pub use scene::Scene;

pub use sdf::{sdf_op, Material, ReflectiveProperties, SdfOutput};

pub use vector::{vec2, vec3, Vec2, Vec3, VecFloat};
