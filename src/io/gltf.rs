use crate::definition::*;
use crate::io::*;
use ::gltf::Gltf;
use std::path::Path;

impl<'a> Loaded<'a> {
    pub fn gltf(
        &'a self,
        path: impl AsRef<Path>,
    ) -> Result<(Vec<CPUMesh>, Vec<CPUMaterial>), IOError> {
        let mut cpu_meshes = Vec::new();
        let mut cpu_materials = Vec::new();

        let bytes = self.bytes(path.as_ref())?;
        let gltf = Gltf::from_slice(bytes)?;
        let (_, buffers, _) = ::gltf::import(path.as_ref())?;
        let base_path = path.as_ref().parent().unwrap();
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                parse_tree(
                    &node,
                    &self,
                    &base_path,
                    &buffers,
                    &mut cpu_meshes,
                    &mut cpu_materials,
                )?;
            }
        }
        Ok((cpu_meshes, cpu_materials))
    }
}

fn parse_tree<'a>(
    node: &::gltf::Node,
    loaded: &'a Loaded,
    path: &Path,
    buffers: &[::gltf::buffer::Data],
    cpu_meshes: &mut Vec<CPUMesh>,
    cpu_materials: &mut Vec<CPUMaterial>,
) -> Result<(), IOError> {
    if let Some(mesh) = node.mesh() {
        let name: String = mesh
            .name()
            .map(|s| s.to_string())
            .unwrap_or(format!("index {}", mesh.index()));
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(read_positions) = reader.read_positions() {
                let mut positions = Vec::new();
                for value in read_positions {
                    positions.push(value[0]);
                    positions.push(value[1]);
                    positions.push(value[2]);
                }

                let normals = reader.read_normals().map(|values| {
                    let mut nors = Vec::new();
                    for value in values {
                        nors.push(value[0]);
                        nors.push(value[1]);
                        nors.push(value[2]);
                    }
                    nors
                });

                let indices = reader.read_indices().map(|values| match values {
                    ::gltf::mesh::util::ReadIndices::U8(iter) => {
                        let mut inds = Vec::new();
                        for value in iter {
                            inds.push(value);
                        }
                        Indices::U8(inds)
                    }
                    ::gltf::mesh::util::ReadIndices::U16(iter) => {
                        let mut inds = Vec::new();
                        for value in iter {
                            inds.push(value);
                        }
                        Indices::U16(inds)
                    }
                    ::gltf::mesh::util::ReadIndices::U32(iter) => {
                        let mut inds = Vec::new();
                        for value in iter {
                            inds.push(value);
                        }
                        Indices::U32(inds)
                    }
                });

                let material = primitive.material();
                let material_name: String = material.name().map(|s| s.to_string()).unwrap_or(
                    material
                        .index()
                        .map(|i| format!("index {}", i))
                        .unwrap_or("default".to_string()),
                );
                let mut parsed = false;
                for material in cpu_materials.iter() {
                    if material.name == material_name {
                        parsed = true;
                        break;
                    }
                }

                if !parsed {
                    let pbr = material.pbr_metallic_roughness();
                    let color = pbr.base_color_factor();
                    let color_texture = if let Some(info) = pbr.base_color_texture() {
                        Some(parse_texture(loaded, path, buffers, info)?)
                    } else {
                        None
                    };
                    let metallic_roughness_texture =
                        if let Some(info) = pbr.metallic_roughness_texture() {
                            Some(parse_texture(loaded, path, buffers, info)?)
                        } else {
                            None
                        };
                    cpu_materials.push(CPUMaterial {
                        name: material_name.clone(),
                        color: Some((color[0], color[1], color[2], color[3])),
                        color_texture,
                        metallic_factor: Some(pbr.metallic_factor()),
                        roughness_factor: Some(pbr.roughness_factor()),
                        metallic_roughness_texture,
                        diffuse_intensity: Some(1.0),
                        specular_intensity: Some(pbr.metallic_factor()),
                        specular_power: Some(pbr.roughness_factor()),
                    });
                }

                let colors = reader.read_colors(0).map(|values| {
                    let mut cols = Vec::new();
                    for value in values.into_rgb_u8() {
                        cols.push(value[0]);
                        cols.push(value[1]);
                        cols.push(value[2]);
                    }
                    cols
                });

                let uvs = reader.read_tex_coords(0).map(|values| {
                    let mut uvs = Vec::new();
                    for value in values.into_f32() {
                        uvs.push(value[0]);
                        uvs.push(value[1]);
                    }
                    uvs
                });

                cpu_meshes.push(CPUMesh {
                    name: name.clone(),
                    positions,
                    normals,
                    indices,
                    colors,
                    uvs,
                    material_name: Some(material_name),
                });
            }
        }
    }

    for child in node.children() {
        parse_tree(&child, loaded, path, buffers, cpu_meshes, cpu_materials)?;
    }
    Ok(())
}

fn parse_texture<'a>(
    loaded: &'a Loaded,
    path: &Path,
    buffers: &[::gltf::buffer::Data],
    info: ::gltf::texture::Info,
) -> Result<CPUTexture<u8>, IOError> {
    let gltf_texture = info.texture();
    let gltf_image = gltf_texture.source();
    let gltf_source = gltf_image.source();
    let tex = match gltf_source {
        ::gltf::image::Source::Uri { uri, .. } => loaded.image(path.join(Path::new(uri)))?,
        ::gltf::image::Source::View { view, .. } => {
            let mut bytes = Vec::with_capacity(view.length());
            bytes.extend(
                (0..view.length())
                    .map(|i| buffers[view.buffer().index()][view.offset() + i])
                    .into_iter(),
            );
            if view.stride() != None {
                unimplemented!();
            }
            image_from_bytes(&bytes)?
        }
    };
    // TODO: Parse sampling parameters
    Ok(tex)
}
