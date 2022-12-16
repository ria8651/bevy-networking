use crate::{despawn_screen, GameState};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContext,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MenuState::default())
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Menu).with_system(menu))
            .add_system_set(
                SystemSet::on_exit(GameState::Menu).with_system(despawn_screen::<InMenu>),
            );
    }
}

#[derive(Component)]
struct InMenu;

#[derive(Resource)]
struct MenuState {
    username: String,
    lobby_name: String,
    lobby_ip: String,
    error: Option<String>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            username: "Bob".to_string(),
            lobby_name: "Epic Lobby".to_string(),
            lobby_ip: "127.0.0.1".to_string(),
            error: None,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), InMenu));
}

fn menu(
    mut egui_context: ResMut<EguiContext>,
    mut menu_state: ResMut<MenuState>,
    mut game_state: ResMut<State<GameState>>,
) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        egui::Area::new("buttons")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.set_width(300.);
                ui.set_height(300.);
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nick:");
                        ui.text_edit_singleline(&mut menu_state.username)
                    });

                    ui.horizontal(|ui| {
                        ui.label("Lobby ip:");
                        ui.text_edit_singleline(&mut menu_state.lobby_ip)
                    });

                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Join").clicked() {
                            if menu_state.username.is_empty() || menu_state.lobby_ip.is_empty() {
                                menu_state.error =
                                    Some("Nick or Lobby ip can't be empty".to_owned());
                            } else {
                                // let server = ChatServer::new(
                                //     menu_state.lobby_name.clone(),
                                //     menu_state.username.clone(),
                                //     menu_state.password.clone(),
                                // );
                                // *state = AppState::HostChat {
                                //     chat_server: Box::new(server),
                                // };

                                game_state.set(GameState::Game).unwrap();
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Lobby name:");
                        ui.text_edit_singleline(&mut menu_state.lobby_name)
                    });

                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Host").clicked() {
                            if menu_state.username.is_empty() || menu_state.lobby_name.is_empty() {
                                menu_state.error =
                                    Some("Nick or Lobby name can't be empty".to_owned());
                            } else {
                                // let server = ChatServer::new(
                                //     menu_state.lobby_name.clone(),
                                //     menu_state.username.clone(),
                                //     menu_state.password.clone(),
                                // );
                                // *state = AppState::HostChat {
                                //     chat_server: Box::new(server),
                                // };

                                game_state.set(GameState::Game).unwrap();
                            }
                        }
                    });

                    if let Some(error) = &menu_state.error {
                        ui.separator();
                        ui.colored_label(Color32::RED, format!("Error: {}", error));
                    }
                });
            });
    });
}
