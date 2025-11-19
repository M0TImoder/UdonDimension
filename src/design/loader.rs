use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_rapier3d::prelude::*;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;
use encoding_rs::SHIFT_JIS;

use crate::robot::drive::DriveInput;
use crate::physics::drag::AirDrag;

#[derive(Event)]
pub struct LoadRobotRequest {
    pub model_name: String,
}

#[derive(Resource)]
struct DeferredLoadRequest {
    model_name: String,
}

#[derive(Component)]
pub struct RobotPart;

#[derive(Component)]
struct PendingJoint {
    parent: Entity,
    data: GenericJoint,
    name: String,
}

#[derive(Component)]
struct PendingCollider {
    scale: Vec3,
}

pub struct RobotLoaderPlugin;

impl Plugin for RobotLoaderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_stl::StlPlugin>() {
            app.add_plugins(bevy_stl::StlPlugin);
        }
        app
            .add_event::<LoadRobotRequest>()
            .add_systems(Update, (
                handle_load_request, 
                load_deferred_robot, 
                apply_mesh_colliders
            ));
    }
}

fn read_file_to_string_smart(path: &Path) -> Option<String> {
    match fs::read(path) {
        Ok(bytes) => {
            if let Ok(utf8_str) = String::from_utf8(bytes.clone()) {
                return Some(utf8_str);
            }
            let (cow, _encoding_used, _had_errors) = SHIFT_JIS.decode(&bytes);
            return Some(cow.into_owned());
        }
        Err(e) => {
            error!("Failed to read file {:?}: {}", path, e);
            None
        }
    }
}

fn handle_load_request(
    mut commands: Commands,
    mut load_events: EventReader<LoadRobotRequest>,
    robot_parts_query: Query<Entity, With<RobotPart>>,
) {
    for event in load_events.read() {
        info!("Request received. Clearing existing robot and scheduling load for: {}", event.model_name);

        for entity in robot_parts_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.insert_resource(DeferredLoadRequest {
            model_name: event.model_name.clone(),
        });
    }
}

fn load_deferred_robot(
    mut commands: Commands,
    deferred_request: Option<Res<DeferredLoadRequest>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let request = match deferred_request {
        Some(r) => r,
        None => return,
    };

    info!("Executing deferred load for: {}", request.model_name);

    let model_dir = Path::new("assets/models").join(&request.model_name);
    
    if !model_dir.exists() {
        error!("Model directory not found: {:?}", model_dir);
        commands.remove_resource::<DeferredLoadRequest>();
        return;
    }

    if let Some(xacro_path) = find_main_xacro(&model_dir) {
        info!("Processing main xacro file: {:?}", xacro_path);
        let urdf_content = convert_xacro_to_urdf_string(&xacro_path);
        
        let urdf_path = xacro_path.with_extension("urdf");
        if let Ok(mut file) = fs::File::create(&urdf_path) {
            let _ = file.write_all(urdf_content.as_bytes());
        }

        match urdf_rs::read_from_string(&urdf_content) {
            Ok(robot) => {
                info!("URDF parsed successfully. Robot name: {}", robot.name);
                spawn_robot_recursive_root(&mut commands, &asset_server, &mut materials, &robot);
            },
            Err(e) => error!("Failed to parse generated URDF: {:?}", e),
        }
    } else {
        error!("No valid .xacro file found in {:?}", model_dir);
    }

    commands.remove_resource::<DeferredLoadRequest>();
}

fn find_main_xacro(root: &Path) -> Option<PathBuf> {
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "xacro" {
                let file_name = path.file_stem().unwrap_or_default().to_string_lossy();
                if !file_name.contains("materials") && !file_name.contains("trans") {
                    return Some(path.to_path_buf());
                }
            }
        }
    }
    None
}

fn convert_xacro_to_urdf_string(path: &Path) -> String {
    let content = match read_file_to_string_smart(path) {
        Some(c) => c,
        None => return String::new(),
    };
    
    let re_find = Regex::new(r"\$\(find\s+([^\)]+)\)").unwrap();
    let resolved_find = re_find.replace_all(&content, |caps: &regex::Captures| {
        let pkg_name = &caps[1];
        format!("assets/models/{}", pkg_name)
    });

    let re_include = Regex::new(r#"<xacro:include\s+filename="([^"]+)"\s*/>"#).unwrap();
    let final_urdf = re_include.replace_all(&resolved_find, |caps: &regex::Captures| {
        let include_path_str = &caps[1];
        let include_path = Path::new(include_path_str);
        
        if let Some(raw) = read_file_to_string_smart(include_path) {
            let resolved = re_find.replace_all(&raw, |caps: &regex::Captures| {
                let pkg_name = &caps[1];
                format!("assets/models/{}", pkg_name)
            });
            let re_xml = Regex::new(r"<\?xml[^>]*\?>").unwrap();
            let no_decl = re_xml.replace(&resolved, "");
            let re_robot_s = Regex::new(r"<\s*robot[^>]*>").unwrap();
            let no_s = re_robot_s.replace(&no_decl, "");
            let re_robot_e = Regex::new(r"<\s*/\s*robot\s*>").unwrap();
            re_robot_e.replace(&no_s, "").to_string()
        } else {
            String::new()
        }
    });
    final_urdf.to_string()
}

fn spawn_robot_recursive_root(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    robot: &urdf_rs::Robot,
) {
    let link_map: HashMap<String, &urdf_rs::Link> = robot.links.iter()
        .map(|l| (l.name.clone(), l))
        .collect();

    let mut child_map: HashMap<String, Vec<(&String, &urdf_rs::Joint)>> = HashMap::new();
    let mut all_children: HashSet<String> = HashSet::new();

    for joint in &robot.joints {
        child_map.entry(joint.parent.link.clone())
            .or_default()
            .push((&joint.child.link, joint));
        all_children.insert(joint.child.link.clone());
    }

    let root_link_name = robot.links.iter()
        .find(|l| !all_children.contains(&l.name))
        .map(|l| &l.name);

    if let Some(root_name) = root_link_name {
        info!("Found root link: {}", root_name);
        let initial_transform = Transform::from_xyz(0.0, 2.0, 0.0);
        
        spawn_link_recursive(
            commands,
            asset_server,
            materials,
            &link_map,
            &child_map,
            root_name,
            initial_transform,
        );
    } else {
        error!("No root link found!");
    }
}

fn spawn_link_recursive(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    link_map: &HashMap<String, &urdf_rs::Link>,
    child_map: &HashMap<String, Vec<(&String, &urdf_rs::Joint)>>,
    link_name: &str,
    transform: Transform,
) -> Entity {
    let link = link_map.get(link_name).expect("Link not found in map");

    let mut entity_cmd = commands.spawn((
        RigidBody::Fixed,
        TransformBundle::from(transform),
        VisibilityBundle::default(),
        Name::new(link.name.clone()),
        RobotPart,
    ));

    if link.inertial.mass.value > 0.0 {
        entity_cmd.insert(AdditionalMassProperties::Mass(link.inertial.mass.value as f32));
    } else {
        entity_cmd.insert(AdditionalMassProperties::Mass(5.0));
    }

    if transform.translation.y >= 1.9 && transform.translation.x == 0.0 && transform.translation.z == 0.0 {
         entity_cmd.insert((
            DriveInput::default(),
            Velocity::default(),
            ExternalForce::default(),
            AirDrag::new(1.0, 0.1, Vec3::splat(0.5)),
        ));
    }

    for visual in &link.visual {
        if let urdf_rs::Geometry::Mesh { filename, scale } = &visual.geometry {
            let mesh_path = filename.replace("package://", "models/").replace("\\", "/");
            let mesh_scale = scale.map_or(Vec3::ONE, |s| Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32));
            let mesh_handle = asset_server.load(&mesh_path);
            let material_handle = materials.add(Color::rgb(0.8, 0.8, 0.8));

            entity_cmd.with_children(|parent| {
                parent.spawn(PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: material_handle,
                    transform: Transform {
                        translation: Vec3::from_array(visual.origin.xyz.map(|v| v as f32)),
                        rotation: Quat::from_euler(
                            EulerRot::XYZ,
                            visual.origin.rpy[0] as f32,
                            visual.origin.rpy[1] as f32,
                            visual.origin.rpy[2] as f32,
                        ),
                        scale: mesh_scale,
                    },
                    ..default()
                });
            });

            entity_cmd.insert(PendingCollider { scale: mesh_scale });
            entity_cmd.insert(mesh_handle);
        }
    }

    let parent_entity = entity_cmd.id();

    if let Some(children) = child_map.get(link_name) {
        for (child_name, joint) in children {
            let joint_offset = Vec3::from_array(joint.origin.xyz.map(|v| v as f32));
            let joint_rotation = Quat::from_euler(
                EulerRot::XYZ,
                joint.origin.rpy[0] as f32,
                joint.origin.rpy[1] as f32,
                joint.origin.rpy[2] as f32,
            );

            let child_transform = transform.mul_transform(Transform {
                translation: joint_offset,
                rotation: joint_rotation,
                ..default()
            });

            let child_entity = spawn_link_recursive(
                commands,
                asset_server,
                materials,
                link_map,
                child_map,
                child_name,
                child_transform,
            );

            let axis = Vec3::from_array(joint.axis.xyz.map(|v| v as f32));
            let joint_data = match joint.joint_type {
                urdf_rs::JointType::Revolute | urdf_rs::JointType::Continuous => {
                    let j = RevoluteJointBuilder::new(axis)
                        .local_anchor1(joint_offset)
                        .local_anchor2(Vec3::ZERO)
                        .build();
                    j.into()
                },
                _ => {
                    let j = FixedJointBuilder::new()
                        .local_anchor1(joint_offset)
                        .local_anchor2(Vec3::ZERO)
                        .build();
                    j.into()
                }
            };

            commands.entity(child_entity).insert(PendingJoint {
                parent: parent_entity,
                data: joint_data,
                name: joint.name.clone(),
            });
        }
    }

    parent_entity
}

fn apply_mesh_colliders(
    mut commands: Commands,
    mut query: Query<(Entity, &Handle<Mesh>, &PendingCollider, Option<&PendingJoint>), With<PendingCollider>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh_handle, pending, pending_joint) in query.iter_mut() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                let scaled_positions: Vec<Vec3> = positions.iter()
                    .map(|p| Vec3::new(p[0], p[1], p[2]) * pending.scale)
                    .collect();
                
                if let Some(collider) = Collider::convex_hull(&scaled_positions) {
                    let robot_collision_group = CollisionGroups::new(Group::GROUP_2, Group::GROUP_1);
                    
                    let mut cmd = commands.entity(entity);
                    cmd
                        .insert(collider)
                        .insert(RigidBody::Dynamic)
                        .insert(robot_collision_group)
                        .remove::<PendingCollider>();

                    if let Some(pj) = pending_joint {
                        info!("Enabling joint: {}", pj.name);
                        cmd.insert(ImpulseJoint::new(pj.parent, pj.data));
                        cmd.insert(Name::new(format!("Joint: {}", pj.name)));
                        cmd.remove::<PendingJoint>();
                    }
                    
                    info!("Generated scaled convex hull for entity {:?}", entity);
                }
            }
        }
    }
}
