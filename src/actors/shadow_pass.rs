use world::{Actor, Handle, World};
use engine::{Asset, Camera, ClearOption, Component, ComponentBased, CullMode, GameObject, Light,
             Material, Mesh, MeshBuffer, MeshData, RenderQueue, RenderTexture, TextureAttachment};
use engine::mesh_util::*;

use world::Processor;

use std::rc::Rc;
use std::sync::Arc;

use math::*;

struct ShadowMap {
    name: String,
    rt: Rc<RenderTexture>,
    light_matrix: Matrix4f,
    light_space_range: (f32, f32),
    partition_z: f32,
    viewport: ((i32, i32), (u32, u32)),

    binder: Option<ShadowMapBinder>,
}

pub struct ShadowPass {
    rt: Rc<RenderTexture>,
    shadow_maps: [ShadowMap; 4],

    light_cache: Option<Arc<Component>>,
    shadow_material: Option<Rc<Material>>,
    light_camera: Camera,

    debug_gameobjects: Vec<Handle<GameObject>>,
    debug_mode: bool,

    use_scene_aabb: bool,
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
// https://stackoverflow.com/questions/43557116/opengl-restore-z-from-depth
// https://gamedev.stackexchange.com/questions/73851/how-do-i-fit-the-camera-frustum-inside-directional-light-space
// https://www.scratchapixel.com/lessons/3d-basic-rendering/perspective-and-orthographic-projection-matrix/orthographic-projection-matrix
// https://msdn.microsoft.com/en-us/library/windows/desktop/ee416307(v=vs.85).aspx
// https://developer.nvidia.com/content/depth-precision-visualized

struct LightMatrixContext {
    inv_pv: Matrix4f,
    view: Matrix4f,
    light_space_scene_aabb: Aabb,

    cam_zfar: f32,
    cam_znear: f32,
}

impl LightMatrixContext {
    fn new(
        light_cam: &Camera,
        light_com: &Arc<Component>,
        world: &World,
    ) -> Option<LightMatrixContext> {
        let cam_borrow = world.current_camera().unwrap();
        let cam = cam_borrow.borrow();

        let p = cam.perspective(world.engine().screen_size);
        let v = cam.v;
        let inv_pv = (p * v).try_inverse().unwrap();

        let light = light_com.try_as::<Light>().unwrap().borrow();
        let lightdir = light.directional().unwrap().direction;
        let mut up = Vector3::y();

        if up.dot(&lightdir.normalize()).abs() > 0.9999 {
            up = Vector3f::z();
        }

        let light_target = Point3 { coords: lightdir };

        let view = Matrix4::look_at_rh(&Point3::new(0.0, 0.0, 0.0), &light_target, &up);

        // Compute scene bound light space aabb.
        // Todo: it is very expensive...
        let scene_bound = world.engine().get_bounds(&light_cam);
        if scene_bound.is_none() {
            return None;
        }

        let light_space_scene_aabb = scene_bound.unwrap().corners().iter().fold(
            Aabb::empty(),
            |mut acc, p| {
                acc.merge_point(&transform_point(&view, &Point3 { coords: *p }).coords);
                acc
            },
        );

        Some(LightMatrixContext {
            cam_znear: cam.znear,
            cam_zfar: cam.zfar,
            inv_pv,
            view,
            light_space_scene_aabb,
        })
    }
}

fn z_to_ndc(z: f32, n: f32, f: f32) -> f32 {
    // a = -(f+n)/(f-n)
    // b = 2fn/(f-n)
    // z_ndc = -a - b / z

    let r = 1.0 / (f - n);
    let a = -(f + n) * r;
    let b = 2.0 * f * n * r;

    return -a - (b / z);
}

fn compute_light_matrix(
    ctx: &LightMatrixContext,
    world: &mut World,
    z_range: &(f32, f32),
    use_scene_aabb: bool,
    debug: Option<&mut Vec<Handle<GameObject>>>,
) -> (Matrix4<f32>, (f32, f32)) {
    let bound_m = ctx.view * ctx.inv_pv;

    // Calculate the 8 corners of the view frustum in world space.

    // This can be done by using the inverse view-projection matrix to
    // transform the 8 corners of the NDC cube (which in OpenGL is [‒1, 1]
    // along each axis).

    // Transform the frustum corners to a space aligned with the shadow map axes.
    // This would commonly be the directional light object's local space.

    // If the camera is perspective, the depth value will be calculated by
    // following equation:
    //
    // a = -(f+n)/(f-n)
    // b = 2fn/(f-n)
    // z_ndc = -a - b / z
    //
    // For example , n = 0.3, f=100.0,  z = 50.0
    // a = -1.0060180
    // b = 0.6018054
    //
    // z_ndc = 1.0060180 - 0.01203611
    //       = 0.99398189

    let (aabb, (nearz, farz)) = if use_scene_aabb {
        (ctx.light_space_scene_aabb, (-1.0, 1.0))
    } else {
        let nearz = z_to_ndc(z_range.0, ctx.cam_znear, ctx.cam_zfar);
        //let farz = 0.99398189;
        let farz = z_to_ndc(z_range.1, ctx.cam_znear, ctx.cam_zfar);

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
        //let mut aabb = ctx.light_space_scene_aabb;

        let mut aabb = Aabb::empty();
        for c in corners.into_iter() {
            aabb.merge_point(&c.coords)
        }

        let max_z = ctx.light_space_scene_aabb.max.z;

        if let Some(debug_gameobjects) = debug {
            add_debug_frustum(
                &ctx.view,
                &[
                    Point3::new(aabb.min.x, aabb.min.y, aabb.min.z),
                    Point3::new(aabb.max.x, aabb.min.y, aabb.min.z),
                    Point3::new(aabb.max.x, aabb.max.y, aabb.min.z),
                    Point3::new(aabb.min.x, aabb.max.y, aabb.min.z),
                    Point3::new(aabb.min.x, aabb.min.y, max_z),
                    Point3::new(aabb.max.x, aabb.min.y, max_z),
                    Point3::new(aabb.max.x, aabb.max.y, max_z),
                    Point3::new(aabb.min.x, aabb.max.y, max_z),
                ],
                world,
                debug_gameobjects,
            );
        }

        aabb.max.z = max_z;
        (aabb, (nearz, farz))
    };

    // build an ortho matrix for directional light

    // Import notes:
    // Don't forget that because we use a right hand coordinate system,
    // the z-coordinates of all points visible by the camera are negative,
    // which is the reason we use -z instead of z.
    let far = -aabb.min.z;
    let near = -aabb.max.z;

    let proj = Matrix4::new_orthographic(aabb.min.x, aabb.max.x, aabb.min.y, aabb.max.y, near, far);

    return (proj * ctx.view, (nearz, farz));
}

struct ShadowMapBinder {
    size: (String, Vector2f),
    light_matrix: (String, Matrix4f),
    range: (String, Vector2f),
    viewport_offset: (String, Vector2f),
    viewport_scale: (String, Vector2f),
}

impl ShadowMap {
    pub fn update_binder(&mut self) {
        let shadow_map_size = self.rt
            .as_texture()
            .size()
            .map(|(w, h)| Vector2::new(w as f32, h as f32))
            .unwrap_or(Vector2::new(0.0, 0.0));

        self.binder = Some(ShadowMapBinder {
            size: (self.name.clone() + ".map_size", shadow_map_size),
            light_matrix: (self.name.clone() + ".light_matrix", self.light_matrix),
            range: (
                self.name.clone() + ".range",
                Vector2::new(self.light_space_range.0, self.light_space_range.1),
            ),
            viewport_offset: (
                self.name.clone() + ".viewport_offset",
                Vector2::new(
                    (self.viewport.0).0 as f32 / shadow_map_size.x,
                    (self.viewport.0).1 as f32 / shadow_map_size.y,
                ),
            ),
            viewport_scale: (
                self.name.clone() + ".viewport_scale",
                Vector2::new(
                    (self.viewport.1).0 as f32 / shadow_map_size.x,
                    (self.viewport.1).1 as f32 / shadow_map_size.y,
                ),
            ),
        });
    }

    pub fn bind(&self, material: &Material) {
        if let Some(ref binder) = self.binder {
            material.set(&binder.size.0, binder.size.1);
            material.set(&binder.light_matrix.0, binder.light_matrix.1);
            material.set(&binder.range.0, binder.range.1);
            material.set(&binder.viewport_offset.0, binder.viewport_offset.1);
            material.set(&binder.viewport_scale.0, binder.viewport_scale.1);
        }
    }

    pub fn render(
        &mut self,
        world: &mut World,
        light_cam: &mut Camera,
        ctx: &LightMatrixContext,
        shadow_material: &Rc<Material>,
        last_partition_z: f32,
        first_render: bool,
        use_scene_aabb: bool,
        debug: Option<&mut Vec<Handle<GameObject>>>,
    ) {
        light_cam.render_texture = Some(self.rt.clone());
        light_cam.rect = Some(self.viewport);

        if world.current_camera().is_none() {
            return;
        }

        let partition = (last_partition_z, self.partition_z);

        let (lm, r) = match debug {
            Some(debug_gameobjects) => {
                for go in debug_gameobjects.iter() {
                    world.remove_game_object(go);
                }
                debug_gameobjects.clear();

                compute_light_matrix(
                    &ctx,
                    world,
                    &partition,
                    use_scene_aabb,
                    Some(debug_gameobjects),
                )
            }

            None => compute_light_matrix(&ctx, world, &partition, use_scene_aabb, None),
        };

        self.light_matrix = lm;
        self.light_space_range = r;

        // if self.debug_mode {
        //     for go in self.debug_gameobjects.iter() {
        //         go.borrow_mut().active = true;
        //     }

        // //world.current_camera().unwrap().borrow_mut().zfar = 1000.0;
        // } else {
        //     for go in self.debug_gameobjects.iter() {
        //         go.borrow_mut().active = false;
        //     }
        //     //world.current_camera().unwrap().borrow_mut().zfar = 100.0;
        // }

        {
            shadow_material.set("uShadowMatrix", self.light_matrix);
        }

        // Render current scene by camera using given frame buffer
        let mut clear_option = ClearOption::default();
        if !first_render {
            clear_option.clear_depth = false;
            clear_option.clear_color = false;
        }

        world.engine_mut().render_pass_with_material(
            light_cam,
            Some(shadow_material),
            clear_option,
        );
    }
}

impl ComponentBased for ShadowPass {}

impl ShadowPass {
    pub fn disable_cascaded(&mut self) {
        self.shadow_maps[0].partition_z = 1000.0;
        self.shadow_maps[0].viewport = ((0, 0), (2048, 2048));

        self.use_scene_aabb = true;
    }

    pub fn set_partitions(&mut self, partitions: &[f32; 4]) {
        self.shadow_maps[0].partition_z = partitions[0];
        self.shadow_maps[1].partition_z = partitions[1];
        self.shadow_maps[2].partition_z = partitions[2];
        self.shadow_maps[3].partition_z = partitions[3];
    }

    fn apply(&self, material: &Material) {
        material.set("uShadowEnabled", true);
        material.set("uShadowMapTexture", self.rt.as_texture());

        for map in self.shadow_maps.iter() {
            map.bind(material);
        }
    }
}

impl Actor for ShadowPass {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let shadow_mat = Material::new(db.new_program("unrust/shadow"));
        self.shadow_material = Some(Rc::new(shadow_mat));

        // Setup proper viewport to render to the whole texture
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
                &AppEvent::KeyUp(ref key) => match key.key.as_str() {
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

            // if still is none, do nothing
            if self.light_cache.is_none() {
                return;
            }
        }

        let ctx = LightMatrixContext::new(
            &self.light_camera,
            &self.light_cache.as_ref().unwrap(),
            world,
        );

        if let Some(ctx) = ctx {
            let mut last_partition_z = self.light_camera.znear;

            if self.use_scene_aabb {
                self.shadow_maps[0].render(
                    world,
                    &mut self.light_camera,
                    &ctx,
                    &self.shadow_material.as_ref().unwrap(),
                    last_partition_z,
                    true,
                    true,
                    if capture {
                        Some(&mut self.debug_gameobjects)
                    } else {
                        None
                    },
                );

                self.shadow_maps[1].light_space_range = (1.0, 1.0);
                self.shadow_maps[2].light_space_range = (1.0, 1.0);
                self.shadow_maps[3].light_space_range = (1.0, 1.0);
            } else {
                for (i, map) in self.shadow_maps.iter_mut().enumerate() {
                    map.render(
                        world,
                        &mut self.light_camera,
                        &ctx,
                        &self.shadow_material.as_ref().unwrap(),
                        last_partition_z,
                        i == 0,
                        false,
                        if capture {
                            Some(&mut self.debug_gameobjects)
                        } else {
                            None
                        },
                    );

                    last_partition_z = map.partition_z;
                }
            }
        }

        // update binder
        {
            for map in self.shadow_maps.iter_mut() {
                map.update_binder();
            }
        }

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
        let texture_size = 2048;
        let texture_size2 = texture_size / 2;
        let rt = Rc::new(RenderTexture::new(
            texture_size,
            texture_size,
            TextureAttachment::Depth,
        ));

        ShadowPass {
            rt: rt.clone(),
            use_scene_aabb: false,
            shadow_maps: [
                ShadowMap {
                    binder: None,
                    name: "uShadowMap[0]".to_owned(),
                    rt: rt.clone(),
                    light_matrix: Matrix4f::identity(),
                    light_space_range: (-1.0, 1.0),
                    partition_z: 50.0,
                    viewport: ((0, 0), (texture_size2, texture_size2)),
                },
                ShadowMap {
                    binder: None,
                    name: "uShadowMap[1]".to_owned(),
                    rt: rt.clone(),
                    light_matrix: Matrix4f::identity(),
                    light_space_range: (-1.0, 1.0),
                    partition_z: 100.0,
                    viewport: ((texture_size2 as i32, 0), (texture_size2, texture_size2)),
                },
                ShadowMap {
                    binder: None,
                    name: "uShadowMap[2]".to_owned(),
                    rt: rt.clone(),
                    light_matrix: Matrix4f::identity(),
                    light_space_range: (-1.0, 1.0),
                    partition_z: 200.0,
                    viewport: ((0, texture_size2 as i32), (texture_size2, texture_size2)),
                },
                ShadowMap {
                    binder: None,
                    name: "uShadowMap[3]".to_owned(),
                    rt: rt.clone(),
                    light_matrix: Matrix4f::identity(),
                    light_space_range: (-1.0, 1.0),
                    partition_z: 1000.0,
                    viewport: ((0, texture_size2 as i32), (texture_size2, texture_size2)),
                },
            ],
            shadow_material: None,
            light_cache: None,
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
