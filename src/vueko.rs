use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    scene::SceneInstance,
};
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::Collider,
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use crate::{assets::ModelAssets, event::Event, event::EventReceiver, fx::crt, VuekoState};

pub struct VuekoPlugin;
impl Plugin for VuekoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(OnEnter(VuekoState::Playing), prepare)
            .add_systems(
                Update,
                (customize_scene_materials, receive_events).run_if(in_state(VuekoState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct ApplyScreenMaterial {}

fn prepare(
    ass: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    models: Res<ModelAssets>,
) {
    // let material = materials.add(StandardMaterial {
    //     base_color_texture: Some(handle.clone()),
    //     unlit: true,
    //     ..default()
    // });
    let size = Extent3d {
        width: 1920,
        height: 1080,
        depth_or_array_layers: 1,
    };

    let handle = images.add(Image::new(
        size,
        TextureDimension::D2,
        vec![255_u8; (size.width * size.height * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    ));

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(handle.clone()),
        unlit: true,
        ..default()
    });

    let cube = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(16.0, 9.0, 16.0)),
                material: material.clone(),
                ..default()
            },
            Collider::cuboid(8.0, 4.5, 8.0),
            RigidBody::Dynamic,
            ApplyScreenMaterial {},
            CaptureMaterial {
                material: material.clone(),
            },
        ))
        .id();

    commands.insert_resource(CaptureImage {
        image: handle.clone(),
    });

    // commands.spawn(CameraFe {
    //     image: handle.clone(),
    //     material: material.clone(),
    // });

    // let cube = commands
    //     .spawn((
    //         SceneBundle {
    //             scene: models.crt.clone(),
    //             transform: Transform::from_xyz(0., 10., 0.),
    //             ..default()
    //         },
    //         ApplyScreenMaterial {},
    //         Collider::cuboid(1.0, 0.6, 0.6),
    //         RigidBody::Dynamic,
    //     ))
    //     .id();
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(100.0, 2.)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(ass.load("textures/rocks.png")),
                normal_map_texture: Some(ass.load("textures/rocks_normal.png")),
                ..default()
            }),
            transform: Transform::from_rotation(Quat::from_rotation_x(0.))
                .with_translation(Vec3::new(0., -10., 0.)),
            ..default()
        },
        Collider::cylinder(1., 20.0),
    ));

    // lights
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 40_000_000.0,
            color: Color::PURPLE,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 20_000_000.0,
            range: 200.,
            color: Color::ALICE_BLUE,
            ..default()
        },
        transform: Transform::from_xyz(-7.0, 6.0, -4.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 20_000_000.0,
            range: 200.,
            color: Color::ALICE_BLUE,
            ..default()
        },
        transform: Transform::from_xyz(2.0, 6.0, 4.0),
        ..default()
    });

    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 0.0, 22.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            crt::Settings { bend: 3.1 },
        ))
        .id();

    commands.entity(cube).add_child(camera);
}

fn receive_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    capture_img: Res<CaptureImage>,
    capture_out: Query<&CaptureMaterial>,
    mut recv: NonSendMut<EventReceiver>,
) {
    let ev = recv.try_recv();

    #[allow(clippy::single_match)]
    match ev {
        Some(Event::Ball { color }) => {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(1.)),
                    material: materials.add(color),
                    ..default()
                },
                Collider::ball(1.),
                RigidBody::Dynamic,
            ));
        }
        Some(Event::CameraFeed { mut frame }) => {
            let Some(image) = images.get_mut(capture_img.image.id()) else {
                return;
            };

            // store the frame in a single image and share this to avoid cloning data
            // for i in 0..image.width() {
            //     for j in 0..image.height() {
            //         frame[0..1].reverse();
            //     }
            // }
            // let mut result = Vec::new();
            // for c in frame.chunks(3) {
            //     result.push(c[0]);
            //     result.push(c[1]);
            //     result.push(c[2]);
            //     result.push(255);
            // }
            image.data = frame;
            // println!("aa");
            for output in capture_out.iter() {
                let Some(material) = materials.get_mut(output.material.id()) else {
                    return;
                };

                // handle needs to be reset for the material to update
                material.base_color_texture = Some(capture_img.image.clone());
            }
        }
        None => (),
    };
}

// fn spin_cube(time: Res<Time>, mut meshes: Query<(&mut Transform, &Screen)>) {
//     for (mut t, _) in meshes.iter_mut() {
//         t.rotation = Quat::from_rotation_y(time.elapsed_seconds_f64() as f32);
//     }
// }

pub fn customize_scene_materials(
    mut commands: Commands,
    unloaded_instances: Query<(Entity, &SceneInstance), With<ApplyScreenMaterial>>,
    handles: Query<(Entity, &Handle<StandardMaterial>)>,
    mut pbr_materials: ResMut<Assets<StandardMaterial>>,
    scene_manager: Res<SceneSpawner>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 1280,
        height: 720,
        depth_or_array_layers: 1,
    };

    let handle = images.add(Image::new(
        size,
        TextureDimension::D2,
        vec![255_u8; (size.width * size.height * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    ));

    for (entity, instance) in unloaded_instances.iter() {
        if scene_manager.instance_is_ready(**instance) {
            commands.entity(entity).remove::<ApplyScreenMaterial>();
        }
        // Iterate over all entities in scene (once it's loaded)
        let mut handles = handles.iter_many(scene_manager.iter_instance_entities(**instance));
        if let Some((entity, material_handle)) = handles.nth(4) {
            let Some(material) = pbr_materials.get_mut(material_handle) else {
                continue;
            };

            material.base_color_texture = Some(handle.clone());
            material.base_color = Color::WHITE;
            material.unlit = true;

            commands.spawn(CaptureMaterial {
                material: material_handle.clone(),
            });
            break;
        }
    }
}

#[derive(Resource)]
pub struct CaptureImage {
    image: Handle<Image>,
}

#[derive(Component)]
pub struct CaptureMaterial {
    material: Handle<StandardMaterial>,
}
