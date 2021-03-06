use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, File};
use engine::render::MeshData;

use obj;
use obj::SimplePolygon;
use std::io::BufReader;

pub struct MeshDataLoader {}
impl Loader<MeshData> for MeshDataLoader {
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<MeshData> {
        let bytes = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
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

        for o in model.objects {
            for g in o.groups {
                for poly in g.polys {
                    for index_tuple in poly {
                        indices.push(index_tuple.0 as u16);
                    }
                }
            }
        }

        Ok(MeshData {
            indices,
            vertices,
            uvs: Some(uvs),
            normals: Some(normals),
            tangents: None,
            bitangents: None,
        })
    }
}

impl Loadable for MeshData {
    type Loader = MeshDataLoader;
}
