use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod game;
mod menu;
mod splash;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Splash,
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Menu)
        .add_plugin(splash::SplashPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(EguiPlugin)
        .run();
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
