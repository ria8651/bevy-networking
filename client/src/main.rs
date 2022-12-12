use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    prelude::*,
};
use bevy_voxel_engine::*;
use character::CharacterEntity;

mod character;
mod networking;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyVoxelEnginePlugin)
        .add_plugin(character::Character)
        .add_plugin(networking::NetworkingPlugin)
        .add_plugin(ui::UiPlugin)
        .add_startup_system(setup)
        .add_system(shoot)
        .add_system(spawn_portals)
        .run();
}

fn setup(mut commands: Commands, mut load_voxel_world: ResMut<LoadVoxelWorld>) {
    // voxel world
    *load_voxel_world = LoadVoxelWorld::File("assets/monu9.vox".to_string());

    // portals
    let portal1 = commands
        .spawn((
            Portal {
                half_size: IVec3::new(0, 0, 0),
                normal: Vec3::new(1.0, 0.0, 0.0),
            },
            Edges {
                material: 120,
                half_size: IVec3::new(0, 0, 0),
            },
            Transform::from_xyz(0.0, 1000.0, 0.0),
        ))
        .id();
    let portal2 = commands
        .spawn((
            Portal {
                half_size: IVec3::new(0, 0, 0),
                normal: Vec3::new(1.0, 0.0, 0.0),
            },
            Edges {
                material: 121,
                half_size: IVec3::new(0, 0, 0),
            },
            Transform::from_xyz(0.0, 1000.0, 0.0),
        ))
        .id();

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
            portal1,
            portal2,
        },
        Velocity::new(Vec3::splat(0.0)),
        BoxCollider {
            half_size: IVec3::new(2, 4, 2),
        },
        BloomSettings::default(),
        Fxaa::default(),
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
        ));
    }
    if input.just_pressed(MouseButton::Right) {
        commands.spawn((
            Transform::from_translation(character.translation),
            Particle { material: 121 },
            Velocity::new(-character.local_z() * 50.0),
            Bullet { bullet_type: 2 },
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
        ));
    }
}

fn spawn_portals(
    mut commands: Commands,
    bullet_query: Query<(&Transform, &Velocity, &Bullet, Entity)>,
    mut character_query: Query<&mut CharacterEntity>,
) {
    for (transform, velocity, bullet, entity) in bullet_query.iter() {
        if bullet.bullet_type == 1 || bullet.bullet_type == 2 {
            if velocity.hit_normal != Vec3::splat(0.0) {
                commands.entity(entity).despawn();

                let normal = velocity.hit_normal;
                let pos = ((transform.translation + normal * (0.5 / VOXELS_PER_METER))
                    * VOXELS_PER_METER)
                    .floor()
                    / VOXELS_PER_METER;

                let plane = (Vec3::splat(1.0) - normal.abs()).as_ivec3();

                let mut character = character_query.single_mut();
                if bullet.bullet_type == 1 {
                    commands.entity(character.portal1).despawn();
                    character.portal1 = commands
                        .spawn((
                            Portal {
                                half_size: plane * 5,
                                normal: normal,
                            },
                            Edges {
                                material: 120,
                                half_size: plane * 6,
                            },
                            Transform::from_xyz(pos.x, pos.y, pos.z),
                        ))
                        .id();
                }
                if bullet.bullet_type == 2 {
                    commands.entity(character.portal2).despawn();
                    character.portal2 = commands
                        .spawn((
                            Portal {
                                half_size: plane * 5,
                                normal: normal,
                            },
                            Edges {
                                material: 121,
                                half_size: plane * 6,
                            },
                            Transform::from_xyz(pos.x, pos.y, pos.z),
                        ))
                        .id();
                }
            }
        }
    }
}