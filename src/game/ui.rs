use super::{character::CharacterEntity, Velocity};
use crate::GameState;
use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa, tonemapping::Tonemapping},
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Slider},
    EguiContext,
};
use bevy_voxel_engine::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_system_set(SystemSet::on_update(GameState::Game).with_system(ui_system));
    }
}

fn ui_system(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    particle_query: Query<Entity, (With<Velocity>, Without<CharacterEntity>)>,
    mut render_graph_settings: ResMut<RenderGraphSettings>,
    mut camera_settings_query: Query<(
        &mut TraceSettings,
        Option<&mut BloomSettings>,
        Option<&mut Tonemapping>,
        Option<&mut Fxaa>,
    )>,
    mut denoise_pass_data: ResMut<DenoiseSettings>,
    diagnostics: Res<Diagnostics>,
    mut game_state: ResMut<State<GameState>>,
) {
    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(egui_context.ctx_mut(), |ui| {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(average) = fps.average() {
                    ui.label(format!("Fps: {:.0}", average));
                }
            }
            for (i, (mut trace_settings, bloom_settings, tonemapping, fxaa)) in
                camera_settings_query.iter_mut().enumerate()
            {
                ui.collapsing(format!("Camera Settings {}", i), |ui| {
                    ui.checkbox(&mut trace_settings.show_ray_steps, "Show ray steps");
                    ui.checkbox(&mut trace_settings.indirect_lighting, "Indirect lighting");
                    ui.add(Slider::new(&mut trace_settings.samples, 1..=8).text("Samples"));
                    ui.add(
                        Slider::new(&mut trace_settings.reprojection_factor, 0.0..=1.0)
                            .text("Reprojection"),
                    );
                    ui.checkbox(&mut trace_settings.shadows, "Shadows");
                    ui.checkbox(&mut trace_settings.misc_bool, "Misc");
                    ui.add(Slider::new(&mut trace_settings.misc_float, 0.0..=1.0).text("Misc"));
                    if let Some(bloom_settings) = bloom_settings {
                        ui.add(
                            Slider::new(&mut bloom_settings.into_inner().intensity, 0.0..=1.0)
                                .text("Bloom"),
                        );
                    }
                    if let Some(tonemapping) = tonemapping {
                        let mut state = match tonemapping.as_ref() {
                            Tonemapping::Enabled { .. } => true,
                            Tonemapping::Disabled => false,
                        };
                        ui.checkbox(&mut state, "Tonemapping");
                        match state {
                            true => {
                                *tonemapping.into_inner() = Tonemapping::Enabled {
                                    deband_dither: true,
                                };
                            }
                            false => {
                                *tonemapping.into_inner() = Tonemapping::Disabled;
                            }
                        }
                    }
                    if let Some(fxaa) = fxaa {
                        ui.checkbox(&mut fxaa.into_inner().enabled, "FXAA");
                    }
                });
            }
            ui.collapsing("Compute", |ui| {
                if ui.button("destroy particles").clicked() {
                    for particle in particle_query.iter() {
                        commands.entity(particle).despawn();
                    }
                }
                ui.label(format!("Particle count: {}", particle_query.iter().count()));
            });
            ui.collapsing("Denoise", |ui| {
                for i in 0..3 {
                    ui.label(format!("Pass {}", i));
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].denoise_strength,
                            0.0..=8.0,
                        )
                        .text("Strength"),
                    );
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].colour_phi,
                            0.01..=1.0,
                        )
                        .text("Colour")
                        .logarithmic(true),
                    );
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].normal_phi,
                            0.1..=100.0,
                        )
                        .text("Normal")
                        .logarithmic(true),
                    );
                    ui.add(
                        Slider::new(
                            &mut denoise_pass_data.pass_settings[i].position_phi,
                            0.01..=1.0,
                        )
                        .text("Position")
                        .logarithmic(true),
                    );
                }
            });
            ui.collapsing("Passes", |ui| {
                ui.checkbox(&mut render_graph_settings.clear, "clear");
                ui.checkbox(&mut render_graph_settings.automata, "automata");
                ui.checkbox(&mut render_graph_settings.animation, "animation");
                ui.checkbox(&mut render_graph_settings.voxelization, "voxelization");
                ui.checkbox(&mut render_graph_settings.rebuild, "rebuild");
                ui.checkbox(&mut render_graph_settings.physics, "physics");
                ui.checkbox(&mut render_graph_settings.trace, "trace");
                ui.checkbox(&mut render_graph_settings.denoise, "denoise");
            });
            if ui.button("Disconnect").clicked() {
                game_state.set(GameState::Menu).unwrap();
            }
        });

    // egui::Window::new("Networking").show(egui_context.ctx_mut(), |ui| {
    //     ui.text_edit_singleline(&mut ui_state.ip);
    //     if ui.button("Connect").clicked() {
    //         connection_events.send(CreateConnectionEvent {
    //             ip: ui_state.ip.clone(),
    //         });
    //     }
    //     if ui.button("Disconnect").clicked() {
    //         disconnect_events.send(DisconnectEvent);
    //     }
    // });
}
