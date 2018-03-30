use engine::asset::{Asset, AssetError, AssetSystem, FileFuture, Resource};
use engine::render::{Material, Mesh, MeshBuffer, MeshData};
use engine::core::Component;
use std::sync::Arc;
use std::borrow::Cow;
use std::path::Path;

use obj;
use obj::SimplePolygon;
use std::io::BufReader;
use std::collections::HashMap;
use std::rc::Rc;
use math::*;

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

#[inline]
fn from_slice_v3(i: usize, s: &[f32]) -> Vector3f {
    return Vector3::new(s[i * 3 + 0], s[i * 3 + 1], s[i * 3 + 2]);
}

#[inline]
fn from_slice_v2(i: usize, s: &[f32]) -> Vector2f {
    return Vector2::new(s[i * 2 + 0], s[i * 2 + 1]);
}

trait CloseToZero {
    fn close_to_zero(&self) -> bool;
}

impl CloseToZero for f32 {
    fn close_to_zero(&self) -> bool {
        self.abs() < 0.0001
    }
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

    let mut i = 0;
    while i < indices.len() {
        let pos1 = from_slice_v3(i, &v_array);
        let pos2 = from_slice_v3(i + 1, &v_array);
        let pos3 = from_slice_v3(i + 2, &v_array);

        let uv1 = from_slice_v2(i, &uv_array);
        let uv2 = from_slice_v2(i + 1, &uv_array);
        let uv3 = from_slice_v2(i + 2, &uv_array);

        let edge1 = pos2 - pos1;
        let edge2 = pos3 - pos1;
        let delta_uv1 = uv2 - uv1;
        let delta_uv2 = uv3 - uv1;

        let d = delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y;
        let tangent: Vector3f;
        let bitangent: Vector3f;

        if !d.close_to_zero() {
            let r = 1.0 / d;
            tangent = (edge1 * delta_uv2.y - edge2 * delta_uv1.y) * r;
            bitangent = (edge2 * delta_uv1.x - edge1 * delta_uv2.x) * r;
        } else {
            // For some ill condition mesh, d would be zero, just fix it
            if !delta_uv1.x.close_to_zero() {
                tangent = edge1 / delta_uv1.x;
            } else if !delta_uv2.x.close_to_zero() {
                tangent = edge2 / delta_uv2.x;
            } else {
                tangent = Vector3::zero();
            }

            if !delta_uv1.y.close_to_zero() {
                bitangent = edge1 / delta_uv1.y;
            } else if !delta_uv2.y.close_to_zero() {
                bitangent = edge2 / delta_uv2.y;
            } else {
                bitangent = Vector3::zero();
            }
        }

        // Triangle share same tangent space
        tangents.extend_from_slice(&tangent[..]);
        tangents.extend_from_slice(&tangent[..]);
        tangents.extend_from_slice(&tangent[..]);

        bittangents.extend_from_slice(&bitangent[..]);
        bittangents.extend_from_slice(&bitangent[..]);
        bittangents.extend_from_slice(&bitangent[..]);

        i += 3;
    }

    for i in 0..indices.len() {
        let n = from_slice_v3(i, &n_array);
        let mut t = from_slice_v3(i, &tangents);
        let b = from_slice_v3(i, &bittangents);

        // Gram-Schmidt orthogonalize
        t = t - n * n.dot(t);

        if n.cross(t).dot(b) < 0.0 {
            t = -t;
        }

        tangents[i * 3 + 0] = t.x;
        tangents[i * 3 + 1] = t.y;
        tangents[i * 3 + 2] = t.z;
    }

    assert!(tangents.len() == v_array.len());
    assert!(bittangents.len() == v_array.len());

    return TangentSpace {
        tangents: Some(tangents),
        bitangents: Some(bittangents),
    };
}

#[derive(Clone, Copy, Debug)]
struct WithNormalMap(bool);

pub struct ObjMaterial {
    pub ambient: Vector3f,
    pub diffuse: Vector3f,
    pub specular: Vector3f,
    pub shininess: f32,
    pub transparent: f32,

    pub diffuse_map: String,
    pub ambient_map: String,
    pub specular_map: String,

    pub alpha_mask: Option<String>,
    pub normal_map: Option<String>,
}

impl Default for ObjMaterial {
    fn default() -> ObjMaterial {
        ObjMaterial {
            ambient: Vector3::new(0.2, 0.2, 0.2),
            diffuse: Vector3::new(1.0, 1.0, 1.0),
            specular: Vector3::new(1.0, 1.0, 1.0),
            shininess: 1000.2,
            transparent: 1.0,
            diffuse_map: "default_white".to_owned(),
            ambient_map: "default_white".to_owned(),
            specular_map: "default_black".to_owned(),
            alpha_mask: None,
            normal_map: None,
        }
    }
}

impl ObjMaterial {
    pub fn from(material: &obj::Material, parent_path: &String) -> ObjMaterial {
        let mut obj_mat = ObjMaterial::default();

        material.ka.map(|ka| obj_mat.ambient = ka.into());
        material.kd.map(|kd| obj_mat.diffuse = kd.into());
        material.ks.map(|ks| obj_mat.specular = ks.into());
        material.ns.map(|ns| obj_mat.shininess = ns);
        material.d.map(|d| obj_mat.transparent = d);
        material
            .map_kd
            .as_ref()
            .map(|map_kd| obj_mat.diffuse_map = parent_path.clone() + &map_kd);
        material
            .map_ka
            .as_ref()
            .map(|map_ka| obj_mat.ambient_map = parent_path.clone() + &map_ka);
        material
            .map_ks
            .as_ref()
            .map(|map_ks| obj_mat.specular_map = parent_path.clone() + &map_ks);
        material
            .map_d
            .as_ref()
            .map(|map_d| obj_mat.alpha_mask = Some(parent_path.clone() + &map_d));

        material
            .map_bump
            .as_ref()
            .map(|map_bump| obj_mat.normal_map = Some(parent_path.clone() + &map_bump));

        obj_mat
    }
}

type MaterialBuilder = Box<Fn(&AssetSystem, &ObjMaterial) -> Rc<Material>>;

struct MaterialCache<A>
where
    A: AssetSystem + Clone + 'static,
{
    map: HashMap<String, (WithNormalMap, Rc<Material>)>,
    parent_path: String,
    asys: A,
    builder: MaterialBuilder,
}

impl<A> MaterialCache<A>
where
    A: AssetSystem + Clone + 'static,
{
    fn new(asys: A, parent_path: String, builder: MaterialBuilder) -> MaterialCache<A> {
        MaterialCache {
            map: HashMap::new(),
            parent_path,
            asys,
            builder,
        }
    }

    fn get_or_insert(&mut self, gm: &obj::Material) -> (WithNormalMap, Rc<Material>) {
        let entry = self.map.get(&gm.name);

        match entry {
            Some(&(b, ref mat)) => (b, mat.clone()),
            None => {
                let r = self.obj_mtl(&gm);
                self.map.insert(gm.name.clone(), (r.0, r.1.clone()));
                r
            }
        }
    }

    fn obj_mtl(&self, gm: &obj::Material) -> (WithNormalMap, Rc<Material>) {
        let obj_mat = ObjMaterial::from(gm, &self.parent_path);

        let material = (*self.builder)(&self.asys, &obj_mat);

        (WithNormalMap(obj_mat.normal_map.is_some()), material)
    }
}

impl PrefabLoader {
    fn load_model<A>(
        asys: A,
        parent: String,
        model: obj::Obj<SimplePolygon>,
        builder: MaterialBuilder,
    ) -> Prefab
    where
        A: AssetSystem + Clone + 'static,
    {
        let vertices: Vec<[f32; 3]> = model.position;
        let uvs: Vec<[f32; 2]> = model.texture;
        let normals: Vec<[f32; 3]> = model.normal;

        // create the mesh componet
        let mut mesh = Mesh::new();

        let mut material_cache = MaterialCache::new(asys.clone(), parent.clone(), builder);

        for o in model.objects {
            for g in o.groups {
                if g.material.is_none() {
                    continue;
                }
                let material = g.material.as_ref().unwrap();
                let (has_normal_map, material) = material_cache.get_or_insert(&material);

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

                let tangent_space = if has_normal_map.0 {
                    compute_tangents(&v_array, &uv_array, &n_array, &indices)
                } else {
                    TangentSpace {
                        tangents: None,
                        bitangents: None,
                    }
                };

                let mesh_data = MeshData {
                    indices: indices,
                    vertices: v_array,
                    uvs: uv_array,
                    tangents: tangent_space.tangents,
                    bitangents: tangent_space.bitangents,
                    normals: n_array,
                };

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

impl Prefab {
    pub fn load_future<A>(
        asys: A,
        objfile: FileFuture,
        builder: MaterialBuilder,
    ) -> Box<Future<Item = Self, Error = AssetError>>
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

            Ok(PrefabLoader::load_model(asys, parent, model, builder))
        });

        // futurize
        Box::new(final_future.map_err(|e| AssetError::FileIoError(e)))
    }
}
