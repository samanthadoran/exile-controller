use std::collections::HashMap;

use config::Config;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Overlay {
    screen_height: f32,
    screen_width: f32,
    show_crosshair: bool,
    show_buttons: bool,
    always_show_overlay: bool,
    windowed_mode: bool,
}

impl Overlay {
    pub fn screen_height(&self) -> f32 {self.screen_height}
    pub fn screen_width(&self) -> f32 {self.screen_width}
    pub fn show_crosshair(&self) -> bool {self.show_crosshair}
    pub fn show_buttons(&self) -> bool {self.show_buttons}
    pub fn always_show_overlay(&self) -> bool {self.always_show_overlay}
    pub fn windowed_mode(&self) -> bool {self.windowed_mode}
}

#[derive(Clone, Deserialize, Debug)]
pub struct Controller {
    controller_deadzone: f32,
    character_x_offset_px: f32,
    character_y_offset_px: f32,
    walk_circle_radius_px: f32,
    close_circle_radius_px: f32,
    mid_circle_radius_px: f32,
    far_circle_radius_px: f32,
    free_mouse_sensitivity_px: f32,
    controller_type: String,
}

impl Controller {
    pub fn controller_deadzone(&self) -> f32 {self.controller_deadzone}
    pub fn character_x_offset_px(&self) -> f32 {self.character_x_offset_px}
    pub fn character_y_offset_px(&self) -> f32 {self.character_y_offset_px}
    pub fn walk_circle_radius_px(&self) -> f32 {self.walk_circle_radius_px}
    pub fn close_circle_radius_px(&self) -> f32 {self.close_circle_radius_px}
    pub fn mid_circle_radius_px(&self) -> f32 {self.mid_circle_radius_px}
    pub fn far_circle_radius_px(&self) -> f32 {self.far_circle_radius_px}
    pub fn free_mouse_sensitivity_px(&self) -> f32 {self.free_mouse_sensitivity_px}
    pub fn controller_type(&self) -> String {self.controller_type.clone()}
}

#[derive(Clone, Deserialize)]
pub struct ApplicationSettings {
    overlay: Overlay,
    button_mapping: HashMap<String, String>,
    ability_mapping: HashMap<String, String>,
    aimable_buttons: Vec<String>,
    action_distances: HashMap<String, String>,
    controller: Controller,
}

impl ApplicationSettings {
    pub fn overlay_settings(&self) -> Overlay {self.overlay.clone()}
    pub fn button_mapping_settings(&self) -> HashMap<String, String> {self.button_mapping.clone()}
    pub fn ability_mapping_settings(&self) -> HashMap<String, String> {self.ability_mapping.clone()}
    pub fn aimable_buttons(&self) -> Vec<String> {self.aimable_buttons.clone()}
    pub fn action_distances(&self) -> HashMap<String, String> {self.action_distances.clone()}
    pub fn controller_settings(&self) -> Controller {self.controller.clone()}
}

pub fn load_settings() -> ApplicationSettings {
    let settings = Config::builder()
                    .add_source(config::File::with_name("settings.toml"))
                    .build()
                    .unwrap_or_else(|error| {
                        panic!("config failed to load. Error: {error}")
                    });
    settings.try_deserialize().unwrap()
}


