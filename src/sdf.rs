use crate::vector::{vec2, vec3, Vec2, Vec3, VecFloat};

#[derive(Clone, Copy)]
pub struct ReflectiveProperties {
    pub ambient_weight: VecFloat,
    pub ao_weight: VecFloat,
    pub visibility_weight: VecFloat,
    pub diffuse_weight: VecFloat,
    pub specular_weight: VecFloat,
    pub specular_exponent: VecFloat,
    pub ao_steps: u32,
    pub ao_step_size: VecFloat,
    pub penumbra: VecFloat,
}

impl ReflectiveProperties {
    pub fn new(
        ambient_weight: VecFloat,
        ao_weight: VecFloat,
        visibility_weight: VecFloat,
        diffuse_weight: VecFloat,
        specular_weight: VecFloat,
        specular_exponent: Option<VecFloat>,
        ao_steps: Option<u32>,
        ao_step_size: Option<VecFloat>,
        penumbra: Option<VecFloat>,
    ) -> ReflectiveProperties {
        ReflectiveProperties {
            ambient_weight,
            ao_weight,
            visibility_weight,
            diffuse_weight,
            specular_weight,
            specular_exponent: specular_exponent.unwrap_or(32.0),
            ao_steps: ao_steps.unwrap_or(5),
            ao_step_size: ao_step_size.unwrap_or(0.01),
            penumbra: penumbra.unwrap_or(48.0),
        }
    }

    pub fn default() -> ReflectiveProperties {
        Self::new(0.1, 0.1, 0.0, 0.8, 1.0, None, None, None, None)
    }

    pub fn lerp(&self, other: &ReflectiveProperties, t: VecFloat) -> ReflectiveProperties {
        fn float_lerp(a: VecFloat, b: VecFloat, t: VecFloat) -> VecFloat {
            a + (b - a) * t
        }
        ReflectiveProperties {
            ambient_weight: float_lerp(self.ambient_weight, other.ambient_weight, t),
            ao_weight: float_lerp(self.ao_weight, other.ao_weight, t),
            visibility_weight: float_lerp(self.visibility_weight, other.visibility_weight, t),
            diffuse_weight: float_lerp(self.diffuse_weight, other.diffuse_weight, t),
            specular_weight: float_lerp(self.specular_weight, other.specular_weight, t),
            specular_exponent: float_lerp(self.specular_exponent, other.specular_exponent, t),
            ao_steps: float_lerp(self.ao_steps as VecFloat, other.ao_steps as VecFloat, t).round()
                as u32,
            ao_step_size: float_lerp(self.ao_step_size, other.ao_step_size, t),
            penumbra: float_lerp(self.penumbra, other.penumbra, t),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Material {
    pub light_source: Vec3,
    pub reflective_properties: ReflectiveProperties,
    pub fg_hsl: Vec3,
    pub bg_hsl: Vec3,
    pub is_shaded: bool,
    pub is_hatched: bool,
}

impl Material {
    pub fn new(
        light_source: &Vec3,
        reflective_properties: Option<ReflectiveProperties>,
        fg_hsl: Option<&Vec3>,
        bg_hsl: Option<&Vec3>,
        is_shaded: bool,
        is_hatched: bool,
    ) -> Material {
        Material {
            light_source: *light_source,
            reflective_properties: reflective_properties
                .unwrap_or_else(ReflectiveProperties::default),
            fg_hsl: *fg_hsl.unwrap_or(&vec3::from_values(0.0, 0.0, 0.0)),
            bg_hsl: *bg_hsl.unwrap_or(&vec3::from_values(0.0, 0.0, 1.0)),
            is_shaded,
            is_hatched,
        }
    }

    pub fn lerp(&self, other: &Material, t: VecFloat) -> Material {
        Material {
            light_source: vec3::lerp(&self.light_source, &other.light_source, t),
            reflective_properties: self
                .reflective_properties
                .lerp(&other.reflective_properties, t),
            fg_hsl: vec3::lerp_hsl(&self.fg_hsl, &other.fg_hsl, t),
            bg_hsl: vec3::lerp_hsl(&self.bg_hsl, &other.bg_hsl, t),
            is_shaded: self.is_shaded || other.is_shaded,
            is_hatched: self.is_hatched || other.is_hatched,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SdfOutput {
    pub distance: VecFloat,
    pub material: Material,
}

impl SdfOutput {
    pub fn new(distance: VecFloat, material: Material) -> SdfOutput {
        SdfOutput {
            distance,
            material: material,
        }
    }

    pub fn min(&self, other: &SdfOutput) -> SdfOutput {
        if self.distance < other.distance {
            *self
        } else {
            *other
        }
    }
}

pub type Sdf = fn(&Vec3) -> SdfOutput;

pub fn op_onion(d: VecFloat, thickness: VecFloat) -> VecFloat {
    d.abs() - thickness
}

// See https://iquilezles.org/articles/smin/
pub fn op_smooth_union(
    dist1: VecFloat,
    dist2: VecFloat,
    smoothing_width: VecFloat,
) -> (VecFloat, VecFloat) {
    let h = (smoothing_width - (dist1 - dist2).abs()).max(0.0) / smoothing_width;
    let mixing = 0.5 * h * h * h;
    let smoothing = (1.0 / 3.0) * mixing * smoothing_width;
    if dist1 < dist2 {
        (dist1 - smoothing, mixing)
    } else {
        (dist2 - smoothing, 1.0 - mixing)
    }
}

pub fn op_smooth_difference(
    dist1: VecFloat,
    dist2: VecFloat,
    smoothing_width: VecFloat,
) -> (VecFloat, VecFloat) {
    let h = (smoothing_width - (dist1 + dist2).abs()).max(0.0) / smoothing_width;
    let mixing = 0.5 * h * h * h;
    let smoothing = (1.0 / 3.0) * mixing * smoothing_width;
    if dist1 > -dist2 {
        (dist1 + smoothing, mixing)
    } else {
        (-dist2 + smoothing, 1.0 - mixing)
    }
}

pub fn op_shift(p: &Vec3, offset: &Vec3) -> Vec3 {
    vec3::sub(p, offset)
}

pub fn op_elongate_y(p: &Vec3, length: VecFloat) -> Vec3 {
    let qy = (p.1.abs() - length).max(0.0);
    vec3::from_values(p.0, qy, p.2)
}

pub fn op_elongate_z(p: &Vec3, length: VecFloat) -> Vec3 {
    let qz = (p.2.abs() - length).max(0.0);
    vec3::from_values(p.0, p.1, qz)
}

pub fn op_rotate_y(p: &Vec3, angle: VecFloat) -> Vec3 {
    let cos_angle = (-angle).cos();
    let sin_angle = (-angle).sin();
    vec3::from_values(
        cos_angle * p.0 + sin_angle * p.2,
        p.1,
        -sin_angle * p.0 + cos_angle * p.2,
    )
}

pub fn op_rotate_z(p: &Vec3, angle: VecFloat) -> Vec3 {
    let cos_angle = (-angle).cos();
    let sin_angle = (-angle).sin();
    vec3::from_values(
        cos_angle * p.0 + sin_angle * p.1,
        -sin_angle * p.0 + cos_angle * p.1,
        p.2,
    )
}

pub fn op_repeat_xz<F>(sdf: F, p: &Vec3, cell_size: &Vec2) -> SdfOutput
where
    F: Fn(&Vec3, &Vec2) -> SdfOutput,
{
    // See https://iquilezles.org/articles/sdfrepetition/
    let p_xz = vec2::from_values(p.0, p.2);
    let cell_id = vec2::round_inplace(vec2::div(&p_xz, cell_size));
    let local_p = vec2::sub(&p_xz, &vec2::mul(&cell_id, cell_size));
    let neighbor_dir = vec2::sign(&local_p);
    [
        vec2::from_values(cell_id.0, cell_id.1 + neighbor_dir.1),
        vec2::from_values(cell_id.0 + neighbor_dir.0, cell_id.1),
        vec2::from_values(cell_id.0 + neighbor_dir.0, cell_id.1 + neighbor_dir.1),
    ]
    .iter()
    .fold(
        sdf(&vec3::from_values(local_p.0, p.1, local_p.1), &cell_id),
        |prev_output, id| {
            let local_p = vec2::sub(&p_xz, &vec2::mul(id, cell_size));
            sdf(&vec3::from_values(local_p.0, p.1, local_p.1), id).min(&prev_output)
        },
    )
}

pub fn op_repeat_finite(p: &Vec3, diameter: &Vec3, repeat_from: &Vec3, repeat_to: &Vec3) -> Vec3 {
    vec3::from_values(
        p.0 - diameter.0 * (p.0 / diameter.0).round().clamp(repeat_from.0, repeat_to.0),
        p.1 - diameter.1 * (p.1 / diameter.1).round().clamp(repeat_from.1, repeat_to.1),
        p.2 - diameter.2 * (p.2 / diameter.2).round().clamp(repeat_from.2, repeat_to.2),
    ) // = p - s * clamp(round(p/s), lim_a, lim_b)
}

pub fn sd_plane(p: &Vec3, normal: &Vec3, offset: VecFloat) -> VecFloat {
    vec3::dot(p, normal) - offset
}

pub fn sd_sphere(p: &Vec3, radius: VecFloat) -> VecFloat {
    vec3::len(p) - radius
}

pub fn sd_box(p: &Vec3, sides: &Vec3) -> VecFloat {
    let q = vec3::from_values(
        p.0.abs() - sides.0,
        p.1.abs() - sides.1,
        p.2.abs() - sides.2,
    ); // q = abs(p) - s
    vec3::len(&vec3::max_float(&q, 0.0)) + q.0.max(q.1).max(q.2).min(0.0) // = length(max(q, 0)) + min(max(q.x, q.y, q.z), 0);
}

pub fn sd_cylinder(p: &Vec3, radius: VecFloat, height: VecFloat) -> VecFloat {
    let len_xz = (p.0 * p.0 + p.2 * p.2).sqrt();
    let d_xz = len_xz - radius;
    let d_y = p.1.abs() - height;
    let d_xz_clamp = d_xz.max(0.0);
    let d_y_clamp = d_y.max(0.0);
    let len_d_clamp = (d_xz_clamp * d_xz_clamp + d_y_clamp * d_y_clamp).sqrt();
    d_xz.max(d_y).min(0.0) + len_d_clamp
}

pub fn sd_cylinder_rounded(
    p: &Vec3,
    radius: VecFloat,
    height: VecFloat,
    offset: VecFloat,
) -> VecFloat {
    sd_cylinder(p, radius - offset, height - offset) - offset
}
