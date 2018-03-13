use world::{Actor, World};
use engine::{ClearOption, Component, ComponentBased, GameObject, Light, Material, RenderTexture,
             Texture, TextureAttachment};
use std::rc::Rc;
use std::sync::Arc;

use math::*;

pub struct ShadowPass {
    rt: Rc<RenderTexture>,
    light_cache: Option<Arc<Component>>,
    shadow_material: Option<Material>,
}

fn compute_light_matrix(com: &Arc<Component>) -> Matrix4<f32> {
    let light = com.try_as::<Light>().unwrap().borrow();
    let lightdir = light.directional().unwrap().direction;

    // build an ortho matrix for directional light
    let proj = Matrix4::new_orthographic(-20.0, 20.0, -20.0, 20.0, -20.0, 40.0);
    let light_target = Point3 { coords: -lightdir };
    let view = Matrix4::look_at_rh(&light_target, &Point3::new(0.0, 0.0, 0.0), &Vector3::y());

    return proj * view;
}

impl ComponentBased for ShadowPass {}

impl ShadowPass {
    pub fn new() -> ShadowPass {
        ShadowPass {
            rt: Rc::new(RenderTexture::new(1024, 1024, TextureAttachment::Depth)),
            shadow_material: None,
            light_cache: None,
        }
    }

    pub fn light_matrix(&self) -> Matrix4<f32> {
        self.light()
            .map(|c| compute_light_matrix(&c))
            .unwrap_or(Matrix4::identity())
    }

    pub fn texture(&self) -> Rc<Texture> {
        self.rt.as_texture().clone()
    }

    fn light(&self) -> Option<&Arc<Component>> {
        self.light_cache.as_ref()
    }

    pub fn apply(&self, material: &Material) {
        let lm = self.light_matrix();
        let shadow_map_size = self.texture()
            .size()
            .map(|(w, h)| Vector2::new(w as f32, h as f32))
            .unwrap_or(Vector2::new(0.0, 0.0));

        material.set("uShadowMatrix", lm);
        material.set("uShadowMapSize", shadow_map_size);
        material.set("uShadowMap", self.texture());
    }
}

impl Actor for ShadowPass {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let shadow_mat = Material::new(db.new_program("unrust/shadow"));
        self.shadow_material = Some(shadow_mat);
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // update light
        if self.light_cache.is_none() {
            self.light_cache = world.engine().find_main_light();
        }

        // Setup fb for camera
        let cam_borrow = world.current_camera().unwrap();
        let mut cam = cam_borrow.borrow_mut();

        {
            self.shadow_material
                .as_ref()
                .unwrap()
                .set("uShadowMatrix", self.light_matrix());
        }

        cam.render_texture = Some(self.rt.clone());

        // Setup proper viewport to render to the whole texture
        cam.rect = Some(((0, 0), (1024, 1024)));

        // Render current scene by camera using given frame buffer
        world.engine().render_pass_with_material(
            &cam,
            self.shadow_material.as_ref(),
            ClearOption::default(),
        );

        // Clean up stuffs in camera, as later we could render normally
        cam.render_texture = None;
        cam.rect = None;
    }
}
