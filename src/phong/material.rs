use crate::core::*;
use crate::definition::*;
use crate::math::*;
use std::rc::Rc;

///
/// The source of color on an object, either a fixed value or a texture.
///
#[derive(Clone)]
pub enum ColorSource {
    Color(Vec4),
    Texture(Rc<Texture2D>),
}

impl std::fmt::Display for ColorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSource::Color(_) => {
                write!(f, "Color")
            }
            ColorSource::Texture(_) => {
                write!(f, "Texture")
            }
        }
    }
}

///
/// A material used for shading an object based on the Phong shading model.
///
#[derive(Clone)]
pub struct PhongMaterial {
    pub name: String,
    pub color_source: ColorSource,
    pub diffuse_intensity: f32,
    pub specular_intensity: f32,
    pub specular_power: f32,
}

impl PhongMaterial {
    ///
    /// Constructor.
    ///
    pub fn new(context: &Context, cpu_material: &CPUMaterial) -> Result<Self, Error> {
        let color_source = if let Some(ref cpu_texture) = cpu_material.color_texture {
            ColorSource::Texture(Rc::new(Texture2D::new(&context, cpu_texture)?))
        } else {
            ColorSource::Color(
                cpu_material
                    .color
                    .map(|(r, g, b, a)| vec4(r, g, b, a))
                    .unwrap_or(vec4(1.0, 1.0, 1.0, 1.0)),
            )
        };
        Ok(Self {
            name: cpu_material.name.clone(),
            color_source,
            diffuse_intensity: cpu_material.diffuse_intensity.unwrap_or(0.5),
            specular_intensity: cpu_material.specular_intensity.unwrap_or(0.2),
            specular_power: cpu_material.specular_power.unwrap_or(6.0),
        })
    }

    pub(crate) fn bind(&self, program: &Program) -> Result<(), Error> {
        program.use_uniform_float("diffuse_intensity", &self.diffuse_intensity)?;
        program.use_uniform_float("specular_intensity", &self.specular_intensity)?;
        program.use_uniform_float("specular_power", &self.specular_power)?;

        match self.color_source {
            ColorSource::Color(ref color) => {
                program.use_uniform_vec4("surfaceColor", color)?;
            }
            ColorSource::Texture(ref texture) => {
                program.use_texture(texture.as_ref(), "tex")?;
            }
        }
        Ok(())
    }
}

impl Default for PhongMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            color_source: ColorSource::Color(vec4(1.0, 1.0, 1.0, 1.0)),
            diffuse_intensity: 0.5,
            specular_intensity: 0.2,
            specular_power: 6.0,
        }
    }
}
