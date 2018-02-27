use engine::asset::loader::{Loadable, Loader};
use engine::asset::{Asset, AssetError, AssetSystem, File, FileFuture, Resource};
use engine::render::{Material, MaterialParam, Mesh, MeshBuffer, MeshData};
use engine::core::Component;
use std::sync::Arc;
use std::rc::Rc;
use std::borrow::Cow;
use na;

use obj;
use obj::SimplePolygon;
use std::io::BufReader;
use std::collections::HashMap;

type Vector3 = na::Vector3<f32>;
use futures::prelude::*;
use futures::future::*;

pub struct Prefab {
    // TODO what is a prefab actually ??
    pub components: Vec<Arc<Component>>,
}

pub struct PrefabLoader {}

impl PrefabLoader {
    fn load_model<A>(asys: A, model: obj::Obj<SimplePolygon>) -> Prefab
    where
        A: AssetSystem + Clone + 'static,
    {
        let vertices: Vec<[f32; 3]> = model.position;
        let uvs: Vec<[f32; 2]> = model.texture;
        let normals: Vec<[f32; 3]> = model.normal;

        let mut ambient = Vector3::new(0.2, 0.2, 0.2);
        let mut diffuse = Vector3::new(0.2, 0.2, 0.2);

        let shader_program = asys.new_program("obj");

        // create the mesh componet
        let mut mesh = Mesh::new();

        for o in model.objects {
            for g in o.groups {
                if let Some(material) = g.material {
                    material.ka.map(|ka| ambient = ka.into());
                    material.kd.map(|kd| diffuse = kd.into());
                }

                let mut indices = Vec::new();
                let mut gv = Vec::new();
                let mut gt = Vec::new();
                let mut gn = Vec::new();

                for poly in g.polys {
                    for index_tuple in poly {
                        indices.push(indices.len() as u16);

                        gv.extend_from_slice(&vertices[index_tuple.0]);

                        index_tuple.1.map(|uv| {
                            gt.extend_from_slice(&uvs[uv]);
                        });

                        index_tuple.2.map(|n| {
                            gn.extend_from_slice(&normals[n]);
                        });
                    }
                }

                let mesh_data = MeshData {
                    indices: indices,
                    vertices: gv,
                    uvs: if gt.len() > 0 { Some(gt) } else { None },
                    normals: if gn.len() > 0 { Some(gn) } else { None },
                };

                let mut params = HashMap::new();
                params.insert(
                    "uMaterial.ambient".to_string(),
                    MaterialParam::Vector3(ambient),
                );

                params.insert(
                    "uMaterial.diffuse".to_string(),
                    MaterialParam::Vector3(diffuse),
                );

                params.insert(
                    "uMaterial.shininess".to_string(),
                    MaterialParam::Float(32.0),
                );

                let material = Material::new(shader_program.clone(), params);

                mesh.add_surface(
                    MeshBuffer::new_from_resource(Resource::new(mesh_data)),
                    Rc::new(material),
                );
            }
        }

        Prefab {
            components: vec![Component::new(mesh)],
        }
    }
}

fn get_mtl_files<A>(asys: A, o: &mut obj::Obj<SimplePolygon>) -> Vec<FileFuture>
where
    A: AssetSystem + Clone + 'static,
{
    let mut files: Vec<FileFuture> = Vec::new();

    for m in &o.material_libs {
        files.push(asys.new_file(m));
    }
    files
}

impl Loadable for Prefab {
    type Loader = PrefabLoader;

    fn load_future<A>(asys: A, objfile: FileFuture) -> Box<Future<Item = Self, Error = AssetError>>
    where
        Self: 'static,
        A: AssetSystem + Clone + 'static,
    {
        let allmat = {
            let asys = asys.clone();
            objfile.and_then(move |mut f| {
                let bytes = f.read_binary()?;
                let mut r = BufReader::new(bytes.as_slice());

                let mut model = obj::Obj::<SimplePolygon>::load_buf(&mut r)?;
                let files = join_all(get_mtl_files(asys, &mut model));

                // attach the model to future
                Ok(files.map(move |x| (x, model)))
            })
        };

        // TODO I don't know why it is needed !!
        let allmat = allmat.and_then(|r| r);

        let final_future = {
            allmat.and_then(move |(files, mut model)| {
                let mut materials = HashMap::new();

                for mut f in files {
                    let bytes = f.read_binary()?;
                    let mtl = obj::Mtl::load(&mut BufReader::new(bytes.as_slice()));
                    for m in mtl.materials {
                        materials.insert(m.name.clone(), Cow::from(m));
                    }
                }

                for object in &mut model.objects {
                    for group in &mut object.groups {
                        if let Some(ref mut mat) = group.material {
                            if let Some(newmat) = materials.get(&mat.name) {
                                *mat = newmat.clone()
                            }
                        }
                    }
                }

                Ok(PrefabLoader::load_model(asys, model))
            })
        };

        // futurize
        Box::new(final_future.map_err(|e| AssetError::FileIoError(e)))
    }
}

impl Loader<Prefab> for PrefabLoader {
    fn load<A: AssetSystem>(_asys: A, mut _file: Box<File>) -> Result<Prefab, AssetError> {
        unimplemented!()
    }
}
