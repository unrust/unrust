use engine::{Asset, Component, GameObject, Material, Mesh, MeshBuffer, RenderQueue, SceneTree};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use super::instance;
use super::label::Label;
use super::widgets;

use engine::IEngine;

struct LabelRenderer {
    go: Option<Rc<RefCell<GameObject>>>,
    mesh: Option<Arc<Component>>,
    material: Option<Rc<Material>>,
}

struct LabelHandle {
    mesh: Arc<Component>,
    mesh_buffer: Option<Rc<MeshBuffer>>,
}

impl Drop for LabelHandle {
    fn drop(&mut self) {
        if let Some(ref mb) = self.mesh_buffer {
            let mesh = self.mesh.try_as::<Mesh>().unwrap();
            mesh.borrow_mut().remove_buffer(&mb);
        }
    }
}

impl LabelRenderer {
    fn new() -> LabelRenderer {
        LabelRenderer {
            go: None,
            material: None,
            mesh: None,
        }
    }

    fn bind(
        &mut self,
        ssize: (u32, u32),
        label: &Label,
        old_handle: Option<&mut LabelHandle>,
        parent: &GameObject,
        engine: &mut IEngine,
    ) -> LabelHandle {
        let material = self.material.get_or_insert_with(|| {
            let db = engine.asset_system();
            let mut material = Material::new(db.new_program("default_ui"));
            material.set("uDiffuse", db.new_texture("default_font_bitmap"));
            material.render_queue = RenderQueue::UI;
            Rc::new(material)
        });

        let go = self.go
            .get_or_insert_with(|| engine.new_game_object(parent));

        let mesh = self.mesh.get_or_insert_with(|| {
            let mesh = Mesh::new();
            let mut gomut = go.borrow_mut();
            gomut.add_component(mesh)
        });

        let hidpi = engine.hidpi_factor();
        let mesh_data = {
            let mut mesh_data = label.bind(ssize, hidpi);
            let disp = widgets::compute_translate(
                &label.pos,
                &label.state.pivot,
                &ssize,
                hidpi,
                &mesh_data.compute_bound().local_aabb(),
            );

            mesh_data.translate(disp);
            mesh_data
        };

        match old_handle {
            Some(h) => {
                if let Some(ref mb) = h.mesh_buffer {
                    let mesh_buffer = mb.clone();
                    h.mesh_buffer = None;
                    mesh_buffer.update_mesh_data(mesh_data);

                    return LabelHandle {
                        mesh: h.mesh.clone(),
                        mesh_buffer: Some(mesh_buffer),
                    };
                }

                LabelHandle {
                    mesh: h.mesh.clone(),
                    mesh_buffer: h.mesh_buffer.clone(),
                }
            }
            None => {
                // MeshBuffer
                let mesh_buffer = MeshBuffer::new(mesh_data);

                // Mesh
                mesh.try_as::<Mesh>()
                    .unwrap()
                    .borrow_mut()
                    .add_surface(mesh_buffer.clone(), material.clone());

                LabelHandle {
                    mesh: mesh.clone(),
                    mesh_buffer: Some(mesh_buffer),
                }
            }
        }
    }
}

enum WidgetHandle {
    GameObject(Rc<RefCell<GameObject>>),
    Label(LabelHandle),
}

type WidgetMap = HashMap<u32, (Rc<widgets::Widget>, WidgetHandle)>;

pub struct Context {
    go: WidgetMap,
    tree: Rc<SceneTree>,
    label_renderer: LabelRenderer,
}

impl Context {
    pub fn new(tree: Rc<SceneTree>) -> Context {
        Context {
            go: HashMap::new(),
            tree,
            label_renderer: LabelRenderer::new(),
        }
    }

    pub fn reset(&mut self) {
        self.label_renderer = LabelRenderer::new();

        self.go.clear()
    }

    pub fn update(&mut self, inner: &instance::ImguiRaw, engine: &mut IEngine) {
        let (sw, sh) = engine.screen_size();

        for w in inner.render_list.iter() {
            let do_insert = {
                let hm = &self.go;
                match hm.get(&w.id()) {
                    None => true,
                    Some(&(ref oldw, _)) => **oldw != **w,
                }
            };

            if do_insert {
                let handle = match w.as_ref() {
                    &widgets::Widget::Label(ref label) => {
                        let h = self.go.get_mut(&w.id()).and_then(|h| match h {
                            &mut (_, WidgetHandle::Label(ref mut h)) => Some(h),
                            _ => None,
                        });

                        WidgetHandle::Label(self.label_renderer.bind(
                            (sw, sh),
                            label,
                            h,
                            &self.tree.root(),
                            engine,
                        ))
                    }
                    &widgets::Widget::Image(ref image) => {
                        WidgetHandle::GameObject(image.bind((sw, sh), &self.tree.root(), engine))
                    }
                };

                self.go.insert(w.id(), (w.clone(), handle));
            }
        }

        // remove all go in hm which id >= last id
        self.go.retain(|k, _| *k <= inner.id);
    }
}
