use bevy::prelude::*;

use super::{despawn_screen, GameState};

// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(GameState::Splash).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Splash).with_system(countdown))
            .add_system_set(
                SystemSet::on_exit(GameState::Splash).with_system(despawn_screen::<InSplash>),
            );
    }
}

#[derive(Component)]
struct InSplash;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");

    // Ui camera
    commands.spawn((Camera2dBundle::default(), InSplash));

    // Display the logo
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    ..default()
                },
                ..default()
            },
            InSplash,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    // This will set the logo to be 200px wide, and auto adjust its height
                    size: Size::new(Val::Px(200.0), Val::Auto),
                    ..default()
                },
                image: UiImage::from(icon),
                ..default()
            });
        });
    
    // Insert the timer as a resource
    commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

// Tick the timer, and change state when finished
fn countdown(
    mut game_state: ResMut<State<GameState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::Menu).unwrap();
    }
}
