use engine::asset::loader::{Loadable, Loader};
use engine::asset::{Asset, AssetError, AssetSystem, File, Resource};
use engine::render::{Material, MaterialParam, Mesh, MeshBuffer, MeshData};
use engine::core::Component;
use std::sync::Arc;
use na;

use obj;
use obj::SimplePolygon;
use std::io::BufReader;
use std::collections::HashMap;

type Vector3 = na::Vector3<f32>;

pub struct Prefab {
    // TODO what is a prefab actually ??
    pub components: Vec<Arc<Component>>,
}

impl Loadable for Prefab {
    type Loader = PrefabLoader;
}

pub struct PrefabLoader {}
impl Loader<Prefab> for PrefabLoader {
    fn load<A: AssetSystem>(asys: A, mut file: Box<File>) -> Result<Prefab, AssetError> {
        let bytes = file.read_binary()
            .map_err(|_| AssetError::InvalidFormat(file.name()))?;
        let mut r = BufReader::new(bytes.as_slice());

        let m = obj::Obj::<SimplePolygon>::load_buf(&mut r);
        let model = m.unwrap();

        let vertices: Vec<f32> = model
            .position
            .into_iter()
            .flat_map(|s| s.to_vec().into_iter())
            .collect();

        let uvs = model
            .texture
            .into_iter()
            .flat_map(|s| s.to_vec().into_iter())
            .collect();

        let normals = model
            .normal
            .into_iter()
            .flat_map(|s| s.to_vec().into_iter())
            .collect();

        let mut indices: Vec<u16> = Vec::new();
        let mut ambient = Vector3::new(0.2, 0.2, 0.2);
        let mut diffuse = Vector3::new(0.2, 0.2, 0.2);

        for o in model.objects {
            for g in o.groups {
                if let Some(material) = g.material {
                    if let Some(ka) = material.ka {
                        ambient = Vector3::new(ka[0], ka[1], ka[2]);
                    }

                    if let Some(kd) = material.kd {
                        diffuse = Vector3::new(kd[0], kd[1], kd[2]);
                    }
                }

                for poly in g.polys {
                    for index_tuple in poly {
                        indices.push(index_tuple.0 as u16);
                    }
                }
            }
        }

        let mesh_data = MeshData {
            indices,
            vertices,
            uvs: Some(uvs),
            normals: Some(normals),
        };

        // create the mesh componet
        let mesh = Component::new(Mesh::new(MeshBuffer::new_from_resource(Resource::new(
            mesh_data,
        ))));

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

        let material = Component::new(Material::new(asys.new_program("obj"), params));

        Ok(Prefab {
            components: vec![mesh, material],
        })
    }
}
