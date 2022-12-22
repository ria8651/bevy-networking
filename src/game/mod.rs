use self::{
    character::{CharacterEntity, CharacterPlugin},
    client::{ClientPlugin, LocalNetworkedEntity},
    networking::NetworkedEntityType,
    server::ServerPlugin,
    ui::UiPlugin,
};
use crate::{despawn_screen, GameState};
use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    prelude::*,
};
use bevy_obj::ObjPlugin;
use bevy_voxel_engine::*;

mod character;
pub mod client;
pub mod networking;
pub mod server;
mod ui;

#[derive(Component)]
struct InGame;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BevyVoxelEnginePlugin)
            .add_plugin(CharacterPlugin)
            .add_plugin(UiPlugin)
            .add_plugin(ClientPlugin)
            .add_plugin(ServerPlugin)
            .add_plugin(ObjPlugin)
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Game).with_system(shoot))
            .add_system_set(SystemSet::on_update(GameState::Game).with_system(spawn_portals))
            .add_system_set(
                SystemSet::on_exit(GameState::Game).with_system(despawn_screen::<InGame>),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    asset_server: Res<AssetServer>,
) {
    // voxel world
    *load_voxel_world = LoadVoxelWorld::File("assets/monu9.vox".to_string());

    // portals
    let mut portals = vec![None; 2];
    for i in 0..2 {
        portals[i] = Some(
            commands
                .spawn((
                    VoxelizationBundle {
                        mesh_handle: asset_server.load("models/portal.obj"),
                        transform: Transform::from_xyz(0.0, 100.0, 0.0)
                            .looking_at(Vec3::ZERO, Vec3::Y)
                            .with_scale(Vec3::new(i as f32 * 2.0 - 1.0, 1.0, i as f32 * 2.0 - 1.0)),
                        voxelization_material: VoxelizationMaterial {
                            flags: Flags::ANIMATION_FLAG | Flags::PORTAL_FLAG,
                            ..default()
                        },
                        ..default()
                    },
                    Portal,
                    InGame,
                    LocalNetworkedEntity {
                        entity_type: NetworkedEntityType::Portal(i as u32),
                    },
                ))
                .with_children(|parent| {
                    // portal border
                    parent.spawn(VoxelizationBundle {
                        mesh_handle: asset_server.load("models/portal_frame.obj"),
                        voxelization_material: VoxelizationMaterial {
                            material: VoxelizationMaterialType::Material(120 + i as u8),
                            flags: Flags::ANIMATION_FLAG | Flags::COLLISION_FLAG,
                        },
                        ..default()
                    });
                })
                .id(),
        );
    }

    // camera
    let transform = Transform::from_xyz(5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        VoxelCameraBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 1.57,
                ..default()
            }),
            ..default()
        },
        CharacterEntity {
            grounded: false,
            look_at: -transform.local_z(),
            up: Vec3::new(0.0, 1.0, 0.0),
            portal1: portals[0].unwrap(),
            portal2: portals[1].unwrap(),
        },
        Velocity::new(Vec3::splat(0.0)),
        BoxCollider {
            half_size: IVec3::new(2, 4, 2),
        },
        BloomSettings::default(),
        Fxaa::default(),
        InGame,
    ));
}

// zero: normal bullet
// one: orange portal bullet
// two: blue portal bullet
#[derive(Component)]
pub struct Bullet {
    bullet_type: u32,
}

fn shoot(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    character: Query<&Transform, With<CharacterEntity>>,
) {
    let character = character.single();

    if input.just_pressed(MouseButton::Left) {
        commands.spawn((
            Transform::from_translation(character.translation),
            Particle { material: 120 },
            Velocity::new(-character.local_z() * 50.0),
            Bullet { bullet_type: 1 },
            LocalNetworkedEntity {
                entity_type: NetworkedEntityType::Bullet(1),
            },
            InGame,
        ));
    }
    if input.just_pressed(MouseButton::Right) {
        commands.spawn((
            Transform::from_translation(character.translation),
            Particle { material: 121 },
            Velocity::new(-character.local_z() * 50.0),
            Bullet { bullet_type: 2 },
            LocalNetworkedEntity {
                entity_type: NetworkedEntityType::Bullet(2),
            },
            InGame,
        ));
    }

    if keyboard.just_pressed(KeyCode::B) {
        commands.spawn((
            Transform::from_translation(character.translation),
            Velocity::new(-character.local_z() * 10.0),
            Bullet { bullet_type: 0 },
            BoxCollider {
                half_size: IVec3::new(3, 3, 3),
            },
            Box {
                material: 14,
                half_size: IVec3::new(3, 3, 3),
            },
            InGame,
        ));
    }
}

fn spawn_portals(
    mut commands: Commands,
    bullet_query: Query<(&Transform, &Velocity, &Bullet, Entity)>,
    mut character_query: Query<&mut CharacterEntity>,
    mut portal_query: Query<&mut Transform, (With<Portal>, Without<Bullet>)>,
) {
    for (transform, velocity, bullet, entity) in bullet_query.iter() {
        if bullet.bullet_type == 1 || bullet.bullet_type == 2 {
            if velocity.hit_normal != Vec3::splat(0.0) {
                commands.entity(entity).despawn();

                let normal = velocity.hit_normal;

                let plane = 1.0 - normal.abs();
                let pos =
                    (transform.translation * plane * VOXELS_PER_METER).floor() / VOXELS_PER_METER;
                let pos = pos + transform.translation * normal.abs();

                let character = character_query.single_mut();
                let entity = match bullet.bullet_type {
                    1 => character.portal1,
                    2 => character.portal2,
                    _ => panic!(),
                };

                let up = if normal.abs() == Vec3::Y {
                    Vec3::Z
                } else {
                    Vec3::Y
                };

                let mut transform = portal_query.get_mut(entity).unwrap();
                transform.translation = pos;
                transform.look_at(pos + normal, up);
            }
        }
    }
}
