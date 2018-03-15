use engine::asset::loader::{Loadable, Loader};
use engine::asset::{Asset, AssetError, AssetResult, AssetSystem, File, FileFuture, Resource};
use engine::render::{Material, Mesh, MeshBuffer, MeshData, RenderQueue, TextureWrap};
use engine::core::Component;
use std::sync::Arc;
use std::borrow::Cow;
use na;
use std::path::Path;

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

fn parent_path(filename: &str) -> String {
    let path = Path::new(filename);
    let parent = path.parent();
    parent
        .map_or("".to_string(), |p| p.to_str().unwrap().to_string() + "/")
        .to_string()
}

pub struct PrefabLoader {}

impl PrefabLoader {
    fn load_model<A>(asys: A, parent: String, model: obj::Obj<SimplePolygon>) -> Prefab
    where
        A: AssetSystem + Clone + 'static,
    {
        let vertices: Vec<[f32; 3]> = model.position;
        let uvs: Vec<[f32; 2]> = model.texture;
        let normals: Vec<[f32; 3]> = model.normal;

        let shader_program = asys.new_program("obj");

        // create the mesh componet
        let mut mesh = Mesh::new();

        for o in model.objects {
            for g in o.groups {
                let mut ambient = Vector3::new(0.2, 0.2, 0.2);
                let mut diffuse = Vector3::new(1.0, 1.0, 1.0);
                let mut specular = Vector3::new(0.2, 0.2, 0.2);
                let mut shininess = 10.0;
                let mut transparent = 1.0;
                let mut diffuse_map = "default_white".to_string();
                let mut ambient_map = "default_white".to_string();
                let mut specular_map = "default_black".to_string();

                if let Some(material) = g.material {
                    material.ka.map(|ka| ambient = ka.into());
                    material.kd.map(|kd| diffuse = kd.into());
                    material.ks.map(|ks| specular = ks.into());
                    material.ns.map(|ns| shininess = ns);
                    material.d.map(|d| transparent = d);
                    material
                        .map_kd
                        .as_ref()
                        .map(|map_kd| diffuse_map = parent.clone() + &map_kd);
                    material
                        .map_ka
                        .as_ref()
                        .map(|map_ka| ambient_map = parent.clone() + &map_ka);
                    material
                        .map_ks
                        .as_ref()
                        .map(|map_ks| specular_map = parent.clone() + &map_ks);
                }

                let mut indices = Vec::new();
                let mut gv = Vec::new();
                let mut gt = Vec::new();
                let mut gn = Vec::new();

                let mut add_v = |index_tuple: obj::IndexTuple| {
                    indices.push(indices.len() as u16);
                    gv.extend_from_slice(&vertices[index_tuple.0]);
                    index_tuple.1.map(|uv| {
                        gt.extend_from_slice(&uvs[uv]);
                    });
                    index_tuple.2.map(|n| {
                        gn.extend_from_slice(&normals[n]);
                    });
                };

                for poly in g.polys {
                    // assert_eq!(poly.len(), 3, "We only handle triangle obj files");
                    match poly.len() {
                        3 => {
                            add_v(poly[0]);
                            add_v(poly[1]);
                            add_v(poly[2]);
                        }
                        4 => {
                            add_v(poly[0]);
                            add_v(poly[1]);
                            add_v(poly[2]);

                            add_v(poly[2]);
                            add_v(poly[3]);
                            add_v(poly[0]);
                        }

                        _ => panic!("We only handle triangle or quad obj files"),
                    }
                }

                let mesh_data = MeshData {
                    indices: indices,
                    vertices: gv,
                    uvs: if gt.len() > 0 { Some(gt) } else { None },
                    normals: if gn.len() > 0 { Some(gn) } else { None },
                };

                let mut material = Material::new(shader_program.clone());

                let ambient_tex = asys.new_texture(&ambient_map);
                ambient_tex.wrap_u.set(TextureWrap::Repeat);
                ambient_tex.wrap_v.set(TextureWrap::Repeat);
                material.set("uMaterial.ambient", ambient);
                material.set("uMaterial.ambient_tex", ambient_tex);

                let mut diffuse_tex = asys.new_texture(&diffuse_map);
                diffuse_tex.wrap_u.set(TextureWrap::Repeat);
                diffuse_tex.wrap_v.set(TextureWrap::Repeat);
                material.set("uMaterial.diffuse", diffuse);
                material.set("uMaterial.diffuse_tex", diffuse_tex);

                let mut specular_tex = asys.new_texture(&specular_map);
                specular_tex.wrap_u.set(TextureWrap::Repeat);
                specular_tex.wrap_v.set(TextureWrap::Repeat);
                material.set("uMaterial.specular", specular);
                material.set("uMaterial.specular_tex", specular_tex);

                material.set("uMaterial.shininess", shininess);
                material.set("uMaterial.transparent", transparent);

                if transparent < 1.0 {
                    material.render_queue = RenderQueue::Transparent;
                }

                mesh.add_surface(
                    MeshBuffer::new_from_resource(Resource::new(mesh_data)),
                    material,
                );
            }
        }

        Prefab {
            components: vec![Component::new(mesh)],
        }
    }
}

fn get_mtl_files<A>(asys: A, basedir: &str, o: &mut obj::Obj<SimplePolygon>) -> Vec<FileFuture>
where
    A: AssetSystem + Clone + 'static,
{
    let mut files: Vec<FileFuture> = Vec::new();

    for m in &o.material_libs {
        files.push(asys.new_file(&(basedir.to_string() + m)));
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
                let parent = parent_path(&f.name());
                let files = join_all(get_mtl_files(asys, &parent, &mut model));

                // attach the model to future
                Ok(files.map(move |x| (x, parent, model)))
            })
        };

        // TODO I don't know why it is needed !!
        let allmat = allmat.and_then(|r| r);

        let final_future = allmat.and_then(move |(files, parent, mut model)| {
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

            Ok(PrefabLoader::load_model(asys, parent, model))
        });

        // futurize
        Box::new(final_future.map_err(|e| AssetError::FileIoError(e)))
    }
}

impl Loader<Prefab> for PrefabLoader {
    fn load<A: AssetSystem>(_asys: A, mut _file: Box<File>) -> AssetResult<Prefab> {
        unimplemented!()
    }
}
