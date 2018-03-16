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
type Vector2 = na::Vector2<f32>;

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

struct TangentSpace {
    tangents: Option<Vec<f32>>,
    bitangents: Option<Vec<f32>>,
}

fn compute_tangents(
    v_array: &Vec<f32>,
    uv_array: &Option<Vec<f32>>,
    n_array: &Option<Vec<f32>>,
    indices: &Vec<u16>,
) -> TangentSpace {
    if uv_array.is_none() || n_array.is_none() {
        return TangentSpace {
            tangents: None,
            bitangents: None,
        };
    }

    let uv_array = uv_array.as_ref().unwrap();
    let n_array = n_array.as_ref().unwrap();

    assert!(v_array.len() % 9 == 0);
    assert!(uv_array.len() % 6 == 0);
    assert!(n_array.len() % 9 == 0);
    assert!(indices.len() % 3 == 0);

    let mut tangents = Vec::with_capacity(v_array.len());
    let mut bittangents = Vec::with_capacity(v_array.len());

    let v_index = |i| i * 3;
    let uv_index = |i| i * 2;
    let n_index = |i| i * 3;

    let mut i = 0;
    while i < indices.len() {
        let pos1 = Vector3::from_row_slice(&v_array[v_index(i + 0)..v_index(i + 1)]);
        let pos2 = Vector3::from_row_slice(&v_array[v_index(i + 1)..v_index(i + 2)]);
        let pos3 = Vector3::from_row_slice(&v_array[v_index(i + 2)..v_index(i + 3)]);

        let uv1 = Vector2::from_row_slice(&uv_array[uv_index(i + 0)..uv_index(i + 1)]);
        let uv2 = Vector2::from_row_slice(&uv_array[uv_index(i + 1)..uv_index(i + 2)]);
        let uv3 = Vector2::from_row_slice(&uv_array[uv_index(i + 2)..uv_index(i + 3)]);

        let edge1 = pos2 - pos1;
        let edge2 = pos3 - pos1;
        let delta_uv1 = uv2 - uv1;
        let delta_uv2 = uv3 - uv1;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tangent = (edge1 * delta_uv2.y - edge2 * delta_uv1.y) * r;
        let bitangent = (edge2 * delta_uv1.x - edge1 * delta_uv2.x) * r;

        // Triangle share same tangent space
        tangents.extend_from_slice(tangent.as_slice());
        tangents.extend_from_slice(tangent.as_slice());
        tangents.extend_from_slice(tangent.as_slice());

        bittangents.extend_from_slice(bitangent.as_slice());
        bittangents.extend_from_slice(bitangent.as_slice());
        bittangents.extend_from_slice(bitangent.as_slice());

        i += 3;
    }

    for i in 0..indices.len() {
        let n = Vector3::from_row_slice(&n_array[n_index(i)..n_index(i + 1)]);
        let mut t = Vector3::from_row_slice(&tangents[n_index(i)..n_index(i + 1)]);
        let b = Vector3::from_row_slice(&bittangents[n_index(i)..n_index(i + 1)]);

        // Gram-Schmidt orthogonalize
        t = t - n * n.dot(&t);

        if n.cross(&t).dot(&b) < 0.0 {
            t = -t;
        }

        tangents[n_index(i) + 0] = t.x;
        tangents[n_index(i) + 1] = t.y;
        tangents[n_index(i) + 2] = t.z;
    }

    assert!(tangents.len() == v_array.len());
    assert!(bittangents.len() == v_array.len());

    return TangentSpace {
        tangents: Some(tangents),
        bitangents: Some(bittangents),
    };
}

impl PrefabLoader {
    fn load_model<A>(asys: A, parent: String, model: obj::Obj<SimplePolygon>) -> Prefab
    where
        A: AssetSystem + Clone + 'static,
    {
        let vertices: Vec<[f32; 3]> = model.position;
        let uvs: Vec<[f32; 2]> = model.texture;
        let normals: Vec<[f32; 3]> = model.normal;

        // create the mesh componet
        let mut mesh = Mesh::new();

        for o in model.objects {
            for g in o.groups {
                let mut ambient = Vector3::new(0.2, 0.2, 0.2);
                let mut diffuse = Vector3::new(1.0, 1.0, 1.0);
                let mut specular = Vector3::new(0.2, 0.2, 0.2);
                let mut shininess = 0.2;
                let mut transparent = 1.0;
                let mut diffuse_map = "default_white".to_string();
                let mut ambient_map = "default_white".to_string();
                let mut specular_map = "default_black".to_string();
                let mut alpha_mask = None;
                let mut normal_map: Option<String> = None;

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
                    material
                        .map_d
                        .as_ref()
                        .map(|map_d| alpha_mask = Some(parent.clone() + &map_d));

                    material
                        .map_bump
                        .as_ref()
                        .map(|map_bump| normal_map = Some(parent.clone() + &map_bump));
                }

                let mut indices = Vec::new();
                let mut v_array = Vec::new();
                let mut uv_array = Vec::new();
                let mut n_array = Vec::new();

                let mut add_v = |index_tuple: obj::IndexTuple| {
                    indices.push(indices.len() as u16);
                    v_array.extend_from_slice(&vertices[index_tuple.0]);
                    index_tuple.1.map(|uv| {
                        uv_array.push(uvs[uv][0]);
                        uv_array.push(1.0 - uvs[uv][1]);
                    });
                    index_tuple.2.map(|n| {
                        n_array.extend_from_slice(&normals[n]);
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

                let uv_array = if uv_array.len() > 0 {
                    Some(uv_array)
                } else {
                    None
                };
                let n_array = if n_array.len() > 0 {
                    Some(n_array)
                } else {
                    None
                };

                let tangent_space = match normal_map {
                    Some(_) => compute_tangents(&v_array, &uv_array, &n_array, &indices),
                    None => TangentSpace {
                        tangents: None,
                        bitangents: None,
                    },
                };

                let mesh_data = MeshData {
                    indices: indices,
                    vertices: v_array,
                    uvs: uv_array,
                    tangents: tangent_space.tangents,
                    bitangents: tangent_space.bitangents,
                    normals: n_array,
                };

                let shader_program = match normal_map {
                    Some(_) => asys.new_program("obj_nm"),
                    None => asys.new_program("obj"),
                };

                let mut material = Material::new(shader_program);

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

                normal_map.as_ref().map(|nm| {
                    let n_tex = asys.new_texture(nm);
                    n_tex.wrap_u.set(TextureWrap::Repeat);
                    n_tex.wrap_v.set(TextureWrap::Repeat);

                    material.set("uMaterial.normal_map", n_tex);
                });

                match alpha_mask {
                    Some(ref f) => material.set("uMaterial.mask_tex", asys.new_texture(&f)),
                    None => material.set("uMaterial.mask_tex", asys.new_texture("default_white")),
                }

                if transparent < 0.9999 || alpha_mask.is_some() {
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
