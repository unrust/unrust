use world::{Actor, Handle, World};
use engine::{Asset, Camera, ClearOption, Component, ComponentBased, CullMode, GameObject, Light,
             Material, Mesh, MeshBuffer, MeshData, RenderQueue, RenderTexture, Texture,
             TextureAttachment};
use engine::mesh_util::*;

use world::Processor;

use std::rc::Rc;
use std::sync::Arc;

use math::*;

pub struct ShadowPass {
    rt: Rc<RenderTexture>,
    light_cache: Option<Arc<Component>>,
    shadow_material: Option<Rc<Material>>,

    light_matrix: Matrix4<f32>,
    light_camera: Camera,

    debug_gameobjects: Vec<Handle<GameObject>>,
    debug_mode: bool,
}

#[repr(usize)]
enum FrustumPoint {
    NLB = 0usize,
    NRB = 1usize,
    NRT = 2usize,
    NLT = 3usize,

    FLB = 4usize,
    FRB = 5usize,
    FRT = 6usize,
    FLT = 7usize,
}

fn add_debug_frustum(
    view: &Matrix4<f32>,
    points: &[Point3<f32>],
    world: &mut World,
    debug_gameobjects: &mut Vec<Handle<GameObject>>,
) {
    let mut wpoints = Vec::new();

    for p in points.iter() {
        let world_p = transform_point(&view.try_inverse().unwrap(), p);
        wpoints.push(world_p.coords);
    }

    let go = world.new_game_object();
    let db = world.asset_system();

    let mut material = Material::new(db.new_program("phong"));
    material.set("uMaterial.diffuse", db.new_texture("default_white"));
    material.set("uMaterial.shininess", 32.0);
    material.states.cull = Some(CullMode::Off);

    let mut mesh = Mesh::new();
    let mut mesh_data = MeshData::default();
    use self::FrustumPoint::*;

    // Near
    mesh_data.add_quad([
        wpoints[NLB as usize],
        wpoints[NRB as usize],
        wpoints[NRT as usize],
        wpoints[NLT as usize],
    ]);

    // Far
    mesh_data.add_quad([
        wpoints[FLB as usize],
        wpoints[FLT as usize],
        wpoints[FRT as usize],
        wpoints[FRB as usize],
    ]);

    // Left
    mesh_data.add_quad([
        wpoints[NLB as usize],
        wpoints[NLT as usize],
        wpoints[FLT as usize],
        wpoints[FLB as usize],
    ]);

    // Right
    mesh_data.add_quad([
        wpoints[NRB as usize],
        wpoints[FRB as usize],
        wpoints[FRT as usize],
        wpoints[NRT as usize],
    ]);

    // Top
    mesh_data.add_quad([
        wpoints[NLT as usize],
        wpoints[NRT as usize],
        wpoints[FRT as usize],
        wpoints[FLT as usize],
    ]);

    // Bottom
    mesh_data.add_quad([
        wpoints[NLB as usize],
        wpoints[FLB as usize],
        wpoints[FRB as usize],
        wpoints[NRB as usize],
    ]);

    mesh.add_surface(MeshBuffer::new(mesh_data), material);
    go.borrow_mut().add_component(mesh);

    debug_gameobjects.push(go);
}

// References:
// https://gamedev.stackexchange.com/questions/73851/how-do-i-fit-the-camera-frustum-inside-directional-light-space
// https://www.scratchapixel.com/lessons/3d-basic-rendering/perspective-and-orthographic-projection-matrix/orthographic-projection-matrix
// https://msdn.microsoft.com/en-us/library/windows/desktop/ee416307(v=vs.85).aspx

fn compute_light_matrix(
    com: &Arc<Component>,
    world: &mut World,
    debug: bool,
    debug_gameobjects: &mut Vec<Handle<GameObject>>,
) -> Matrix4<f32> {
    let cam_borrow = world.current_camera().unwrap();
    let cam = cam_borrow.borrow();

    let p = cam.perspective(world.engine().screen_size);
    let v = cam.v;
    let inv_pv = (p * v).try_inverse().unwrap();

    let light = com.try_as::<Light>().unwrap().borrow();
    let lightdir = light.directional().unwrap().direction;
    let light_target = Point3 { coords: lightdir };
    let view = Matrix4::look_at_rh(&Point3::new(0.0, 0.0, 0.0), &light_target, &Vector3::y());

    let bound_m = view * inv_pv;

    // Calculate the 8 corners of the view frustum in world space.

    // This can be done by using the inverse view-projection matrix to
    // transform the 8 corners of the NDC cube (which in OpenGL is [‒1, 1]
    // along each axis).

    // Transform the frustum corners to a space aligned with the shadow map axes.
    // This would commonly be the directional light object's local space.

    // If the camera is perspective, the depth value will be calculated by
    // following equation:
    //
    // a = -(n+f)/(n-f)
    // b = 2fn(n-f)
    // z_ndc = a - b / z
    //
    // For example , n = 0.3, f=100.0,  z = 50.0
    // a = -1.0060180
    // b = 0.6018054
    //
    // z_ndc = 1.0060180 - 0.01203611
    //       = 0.99398189

    let nearz = -1.0;
    //let farz = 0.99398189;
    let farz = 1.0;

    let corners = [
        transform_point(&bound_m, &Point3::new(-1.0, -1.0, nearz)),
        transform_point(&bound_m, &Point3::new(1.0, -1.0, nearz)),
        transform_point(&bound_m, &Point3::new(1.0, 1.0, nearz)),
        transform_point(&bound_m, &Point3::new(-1.0, 1.0, nearz)),
        transform_point(&bound_m, &Point3::new(-1.0, -1.0, farz)),
        transform_point(&bound_m, &Point3::new(1.0, -1.0, farz)),
        transform_point(&bound_m, &Point3::new(1.0, 1.0, farz)),
        transform_point(&bound_m, &Point3::new(-1.0, 1.0, farz)),
    ];

    // light local space aabb
    let mut aabb = Aabb::empty();
    for c in corners.into_iter() {
        aabb.merge_point(&c.coords)
    }

    // Compute scene bound light space aabb.
    // Todo: it is very expensive...
    let scene_bound = world.engine().get_bounds(&cam);
    let light_space_scene_bounds = scene_bound.corners().iter().fold(
        Aabb::empty(),
        |mut acc, p| {
            acc.merge_point(&transform_point(&view, &Point3 { coords: *p }).coords);
            acc
        },
    );

    if debug {
        add_debug_frustum(
            &view,
            &[
                Point3::new(aabb.min.x, aabb.min.y, light_space_scene_bounds.min.z),
                Point3::new(aabb.max.x, aabb.min.y, light_space_scene_bounds.min.z),
                Point3::new(aabb.max.x, aabb.max.y, light_space_scene_bounds.min.z),
                Point3::new(aabb.min.x, aabb.max.y, light_space_scene_bounds.min.z),
                Point3::new(aabb.min.x, aabb.min.y, light_space_scene_bounds.max.z),
                Point3::new(aabb.max.x, aabb.min.y, light_space_scene_bounds.max.z),
                Point3::new(aabb.max.x, aabb.max.y, light_space_scene_bounds.max.z),
                Point3::new(aabb.min.x, aabb.max.y, light_space_scene_bounds.max.z),
            ],
            world,
            debug_gameobjects,
        );
    }

    // build an ortho matrix for directional light

    // Import notes:
    // Don't forget that because we use a right hand coordinate system,
    // the z-coordinates of all points visible by the camera are negative,
    // which is the reason we use -z instead of z.

    let proj = Matrix4::new_orthographic(
        aabb.min.x,
        aabb.max.x,
        aabb.min.y,
        aabb.max.y,
        -light_space_scene_bounds.max.z,
        -light_space_scene_bounds.min.z,
    );

    return proj * view;
}

impl ComponentBased for ShadowPass {}

impl ShadowPass {
    pub fn texture(&self) -> Rc<Texture> {
        self.rt.as_texture()
    }

    fn light(&self) -> Option<&Arc<Component>> {
        self.light_cache.as_ref()
    }

    pub fn apply(&self, material: &Material) {
        let lm = self.light_matrix;
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
        self.shadow_material = Some(Rc::new(shadow_mat));

        self.light_camera.render_texture = Some(self.rt.clone());

        // Setup proper viewport to render to the whole texture
        self.light_camera.rect = Some(((0, 0), (1024, 1024)));
        self.light_camera.enable_frustum_culling = false;
        self.light_camera.included_render_queues = Some(Default::default());
        self.light_camera
            .included_render_queues
            .as_mut()
            .unwrap()
            .insert(RenderQueue::Opaque);
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        use uni_app::AppEvent;
        let mut capture = false;

        for evt in world.events().iter() {
            match evt {
                &AppEvent::KeyUp(ref key) => match key.code.as_str() {
                    "Space" => {
                        capture = true;
                    }
                    "KeyO" => {
                        self.debug_mode = !self.debug_mode;
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        // update light
        if self.light_cache.is_none() {
            self.light_cache = world.engine().find_main_light();
        }

        if world.current_camera().is_none() {
            return;
        }

        // Update light matrix
        let light = self.light().cloned();

        if let Some(c) = light {
            if capture {
                for go in self.debug_gameobjects.iter() {
                    world.remove_game_object(go);
                }
                self.debug_gameobjects.clear();

                // Change camera zfar more to know where is the end of frustum
                //world.current_camera().unwrap().borrow_mut().zfar = 100.0;

                self.light_matrix =
                    compute_light_matrix(&c, world, true, &mut self.debug_gameobjects);
                self.debug_mode = true;
            } else {
                self.light_matrix =
                    compute_light_matrix(&c, world, false, &mut self.debug_gameobjects);
            }
        }

        if self.debug_mode {
            for go in self.debug_gameobjects.iter() {
                go.borrow_mut().active = true;
            }

        //world.current_camera().unwrap().borrow_mut().zfar = 1000.0;
        } else {
            for go in self.debug_gameobjects.iter() {
                go.borrow_mut().active = false;
            }
            //world.current_camera().unwrap().borrow_mut().zfar = 100.0;
        }

        {
            self.shadow_material
                .as_ref()
                .unwrap()
                .set("uShadowMatrix", self.light_matrix);
        }

        // Render current scene by camera using given frame buffer
        world.engine_mut().render_pass_with_material(
            &self.light_camera,
            self.shadow_material.as_ref(),
            ClearOption::default(),
        );

        // GUI
        use imgui;
        use imgui::Metric::*;
        imgui::pivot((0.0, 1.0));
        let mut mat = Material::new(world.asset_system().new_program("unrust/shadow_display"));
        mat.set("uDepthMap", self.rt.as_texture());
        mat.render_queue = RenderQueue::UI;

        imgui::image_with_material(Native(0.0, 1.0), Pixel(100.0, 100.0), Rc::new(mat));
    }
}

impl Processor for ShadowPass {
    fn new() -> ShadowPass {
        ShadowPass {
            rt: Rc::new(RenderTexture::new(1024, 1024, TextureAttachment::Depth)),
            shadow_material: None,
            light_cache: None,
            light_matrix: Matrix4::identity(),
            light_camera: Camera::new(),
            debug_gameobjects: Vec::new(),
            debug_mode: false,
        }
    }

    fn apply_materials(&self, materials: &Vec<Rc<Material>>) {
        for m in materials.iter() {
            self.apply(&m);
        }
    }

    fn watch_material() -> bool
    where
        Self: Sized,
    {
        return true;
    }
}
