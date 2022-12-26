use std::collections::HashMap;

use crate::settings:: ApplicationSettings;

use super::input::{ControllerButton, AnalogStick};
use super::action_handler::{ActionHandler, ActionType};

#[derive(PartialEq)]
pub enum ActionDistance {
    Close,
    Mid,
    Far,
    None,
}

struct PlannedAction {
    name: String,
    just_pressed: bool,
    aimable: bool,
    distance: ActionDistance,
}

pub struct ActionManager {
    action_handler: ActionHandler,
    planned_actions: Vec<PlannedAction>,
    settings: ApplicationSettings,
    holding_walk: bool,
    walking_angle: f32,
    holding_aim: bool,
    aiming_angle: f32,
    aiming_stick_direction: Vec<f32>,
    holding_ability: bool,
}

impl ActionManager {
    pub fn initialize (application_settings: ApplicationSettings) -> ActionManager {
        ActionManager {
            action_handler: ActionHandler::default(),
            planned_actions: Vec::<PlannedAction>::with_capacity(application_settings.button_mapping_settings().keys().count()), 
            settings: application_settings,
            holding_walk: false,
            walking_angle: 0.0,
            holding_aim: false,
            aiming_angle: 0.0,
            aiming_stick_direction: vec![0.0, 0.0],
            holding_ability: false,
        }
    }

    pub fn process_input_buttons(&mut self, named_controller_buttons: HashMap<String, &mut ControllerButton>) {
        for (action_name, button) in named_controller_buttons {
            if button.just_pressed && button.just_unpressed {
                panic!("This should never be possible!")
            }
            if button.just_pressed {
                println!("Just pressed {:?} ", action_name);
                let can_be_aimed = self.settings.aimable_buttons().contains(&action_name);
                let action_distance = self.get_ability_action_distance(&action_name);
                self.planned_actions.push(PlannedAction {name: action_name, 
                                                        just_pressed: true, 
                                                        aimable: can_be_aimed,
                                                        distance: action_distance,
                                                    });
                button.just_pressed = false;
            }
            else if button.just_unpressed {
                println!("Just unpressed {:?} ", action_name);
                self.planned_actions.push(PlannedAction {name: action_name, 
                                                        just_pressed: false, 
                                                        aimable: false, // Don't need this for unpress
                                                        distance: ActionDistance::None, // Don't need this for unpress
                });
                button.just_unpressed = false;
            }
        }

    }
    pub fn process_input_analogs(&mut self, left_stick: AnalogStick, right_stick: AnalogStick) {
        if !left_stick.joystick_in_deadzone() {
            self.holding_walk = true;
            self.walking_angle = left_stick.stick_angle();
        } else {
            self.holding_walk = false;
        }
        if !right_stick.joystick_in_deadzone() {
            self.holding_aim = true;
            self.aiming_angle = right_stick.stick_angle();
            self.aiming_stick_direction = right_stick.stick_direction();
        }else {
            self.holding_aim = false;
        }
    }

    pub fn handle_character_actions(&mut self, ctx: &egui::Context) {
        // Execute planned actions
        while let Some(planned_action) = self.planned_actions.pop() {
            let key_name = self.settings.button_mapping_settings().get(&planned_action.name).unwrap().to_string();
            // println!("{key_name}");
            if planned_action.just_pressed {
                if planned_action.aimable {
                    if self.holding_walk && self.holding_aim {
                        let (new_x, new_y) = self.get_radial_location(self.get_attack_circle_radius(planned_action.distance), self.aiming_angle);
                        self.action_handler.move_mouse(new_x as f64, new_y as f64);
                    } else if self.holding_walk && !self.holding_aim {
                        let (new_x, new_y) = self.get_radial_location(self.get_attack_circle_radius(planned_action.distance), self.walking_angle);
                        self.action_handler.move_mouse(new_x as f64, new_y as f64);
                    }
                    // todo probably inject a delay for the two above
                } else if planned_action.distance != ActionDistance::None && self.holding_walk {
                        let (new_x, new_y) = self.get_radial_location(self.get_attack_circle_radius(planned_action.distance), self.walking_angle);
                        self.action_handler.move_mouse(new_x as f64, new_y as f64);
                }
                self.action_handler.handle_action(ActionType::Press, key_name);
            } else {
                self.action_handler.handle_action(ActionType::Release, key_name);
            }
        }

        // if we're holding an ability but didn't just press something, we need the cursor to swivel if we're also holding a stick.
        // This block accomplishes that swivel, prioritizing aiming_angle if any held buttons are aimable, and targeting the longest distance
        // If none of the held abilities are aimable or have preset distances, this causes the cursor to snap to the walking circle if held.
        // If none of the held abilities are aimable or have preset distances, AND we're not walking, this lets you free-aim the ability with right stick
        self.holding_ability = self.action_handler.is_ability_key_held();

        if self.holding_ability && self.holding_walk {
            let held_ability_actions: Vec<String> = self.action_handler.get_held_ability_actions()
                                                                        .iter()
                                                                        .map(|key| self.settings.ability_mapping_settings().get(key).unwrap().to_owned())
                                                                        .collect();
            let held_abilities_with_action_distance_set = held_ability_actions.into_iter()
                                                                                    .filter(|action| self.get_ability_action_distance(action) != ActionDistance::None);
            let mut some_held_action_aimable = false;
            let mut chosen_distance =  0.0;
            // println!("{:?}", held_abilities_with_action_distance_set.clone());
            // println!("{:?}", held_abilities_with_action_distance_set.clone().map(|action| (action.to_owned(), self.get_attack_circle_radius(self.get_ability_action_distance(&action)))) );
            for (action, distance) in held_abilities_with_action_distance_set.map(|action| (action.to_owned(), self.get_attack_circle_radius(self.get_ability_action_distance(&action)))) {
                // println!("{:?}, {:?}", action, distance);
                if self.settings.aimable_buttons().contains(&action) {
                    some_held_action_aimable = true;
                }
                if distance > chosen_distance {
                    chosen_distance = distance;
                }
            }
            // println!("{:?}, {:?}", some_held_action_aimable, chosen_distance);
            if chosen_distance == 0.0 {
                // no held ability had a preset distance, use walking distance
                chosen_distance = self.settings.controller_settings().walk_circle_radius_px();
            }
            if some_held_action_aimable && self.holding_aim {
                let (new_x, new_y) = self.get_radial_location(chosen_distance, self.aiming_angle);
                self.action_handler.move_mouse(new_x as f64, new_y as f64);
            } else {
                let (new_x, new_y) = self.get_radial_location(chosen_distance, self.walking_angle);
                self.action_handler.move_mouse(new_x as f64, new_y as f64);
            }
        }
        
        // if aiming and not moving!
        if self.holding_aim && !self.holding_walk {
            let (new_x_pos, new_y_pos) = self.get_free_move_update(ctx);
            self.action_handler.move_mouse(new_x_pos, new_y_pos);
        }

        // if moving!
        if self.holding_walk && !self.holding_ability {
            let (new_x, new_y) = self.get_radial_location(self.settings.controller_settings().walk_circle_radius_px(), self.walking_angle);
            self.action_handler.move_mouse(new_x as f64, new_y as f64);
            self.action_handler.handle_action(ActionType::Press, "LeftClick".to_string());
        } else {
            self.action_handler.handle_action(ActionType::Release, "LeftClick".to_string());
        }
  
    }

    fn get_radial_location(&self, circle_radius: f32, angle: f32) -> (f32, f32) {
        let screen_adjustment_x = angle.cos() * circle_radius;
        let screen_adjustment_y = angle.sin() * circle_radius;
        let new_x = self.settings.overlay_settings().screen_width()/2.0 + screen_adjustment_x + self.settings.controller_settings().character_x_offset_px();
        let new_y = self.settings.overlay_settings().screen_height()/2.0 - screen_adjustment_y - self.settings.controller_settings().character_y_offset_px();
        (new_x, new_y)
    }

    fn get_attack_circle_radius(&self, action_distance: ActionDistance) -> f32 {
        match action_distance {
            ActionDistance::Close => {self.settings.controller_settings().close_circle_radius_px()},
            ActionDistance::Mid => {self.settings.controller_settings().mid_circle_radius_px()},
            ActionDistance::Far => {self.settings.controller_settings().far_circle_radius_px()},
            _ => {self.settings.controller_settings().walk_circle_radius_px()}
        }
    }

    fn get_free_move_update(&self, ctx: &egui::Context) -> (f64, f64){
        let screen_adjustment_x = self.aiming_stick_direction[0] * self.settings.controller_settings().free_mouse_sensitivity_px();
        let screen_adjustment_y = -1.0 * self.aiming_stick_direction[1] * self.settings.controller_settings().free_mouse_sensitivity_px();
        
        // There is a chance that there _is_ no mouse position.
        match ctx.input().pointer.hover_pos() {
            Some(position) => ((position.x + screen_adjustment_x) as f64, (position.y + screen_adjustment_y) as f64),
            // Should we just panic here?
            None => (0.0f64, 0.0f64),
        }
    }

    fn get_ability_action_distance(&self, name: &String) -> ActionDistance {
        if self.settings.action_distances().contains_key(name) {
            // println!("herp {:?}", name);
            match self.settings.action_distances().get(name).unwrap().as_str() {
                "close" => {ActionDistance::Close},
                "mid" => {ActionDistance::Mid},
                "far" => {ActionDistance::Far},
                _ => {ActionDistance::None}
            }
        } else {
            // println!("derp {:?}", name);
            ActionDistance::None}
    } 
}




