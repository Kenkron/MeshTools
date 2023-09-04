use std::fmt::Display;

use egui::{Button, DragValue};
use glm::{Vec3, Mat4};

pub trait Transformation: Display{
    fn matrix(&self) -> Mat4;
    fn ui(&mut self, ui: &mut egui::Ui);
}

fn vec3_control(ui: &mut egui::Ui, vector: &mut Vec3) {
    ui.horizontal(|ui| {
        ui.label("X");
        ui.add(DragValue::new(&mut vector.x));
        ui.label("Y");
        ui.add(DragValue::new(&mut vector.y));
        ui.label("Z");
        ui.add(DragValue::new(&mut vector.z));
    });
}

pub struct Rotation {
    axis: Vec3,
    degrees: f32
}
impl Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rotate {}°", self.degrees)
    }
}
impl Transformation for Rotation {
    fn matrix(&self) -> Mat4 {
        return Mat4::new_rotation(self.axis.normalize() * self.degrees.to_radians());
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Axis");
        ui.horizontal(|ui| {
            vec3_control(ui, &mut self.axis);
        });
        ui.label("Angle");
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.degrees));
            ui.label("°")
        });
    }
}

pub struct Scale {
    scale: Vec3
}
impl Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scale {}, {}, {}", self.scale.x, self.scale.y, self.scale.z)
    }
}
impl Transformation for Scale {
    fn matrix(&self) -> Mat4 {
        Mat4::new_nonuniform_scaling(&self.scale)
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        vec3_control(ui, &mut self.scale);
    }
}
pub struct Translation {
    translation: Vec3
}
impl Display for Translation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let translation = self.translation;
        write!(f, "Translation {}, {}, {}", translation.x, translation.y, translation.z)
    }
}
impl Transformation for Translation {
    fn matrix(&self) -> Mat4 {
        Mat4::new_translation(&self.translation)
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        vec3_control(ui, &mut self.translation);
    }
}

pub struct TransformationUI {
    pub transformations: Vec::<Box<dyn Transformation>>,
    pub selection: Option<usize>
}

impl TransformationUI {
    pub fn new() -> Self {
        return Self {
            transformations: Vec::<Box<dyn Transformation>>::new(),
            selection: None
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.set_max_width(200.0);
        for i in 0..self.transformations.len() {
            let mut removed = Option::<usize>::None;
            ui.horizontal(|ui| {
                if ui.button("×").clicked() {
                    removed = Some(i);
                    return;
                }
                if ui.button("edit").clicked() {
                    self.selection = Some(i);
                }
                if ui.add_enabled(i > 0, Button::new("^")).clicked() {
                    self.transformations.swap(i, i-1);
                    self.selection = match self.selection {
                        Some(selection) => {
                            if selection == i {
                                print!("Sub");
                                Some(i - 1)
                            } else if selection == i - 1 {
                                print!("Add");
                                Some(i)
                            } else {
                                print!("Same");
                                Some(selection)
                            }
                        },
                        None => {print!("None"); None},
                    };
                    println!(" {}", i);
                }
            });
            if let Some(i) = removed {
                self.transformations.remove(i);
                self.selection = match self.selection {
                    Some(selection) => {
                        if selection == i {
                            None
                        } else if selection > i {
                            Some(selection - 1)
                        } else {
                            Some(selection)
                        }
                    },
                    None => {None},
                };
                continue;
            }
            ui.label(self.transformations[i].to_string());
            if let Some(selection) = self.selection {
                if selection == i {
                    self.transformations[i].ui(ui);
                }
            }
            ui.separator();
        }
        ui.menu_button("+", |ui| {
            if ui.button("Rotation").clicked() {
                self.transformations.push(
                    Box::new(Rotation{axis: *Vec3::z_axis(), degrees: 0.0}));
            }
            if ui.button("Scale").clicked() {
                self.transformations.push(
                    Box::new(Scale{scale: Vec3::new(1.,1.,1.)}));
            }
            if ui.button("Translation").clicked() {
                self.transformations.push(
                    Box::new(Translation{translation: Vec3::zeros()}));
            }
        });
    }
    pub fn get_matrix(&self) -> Mat4{
        let mut result = Mat4::identity();
        for t in &self.transformations {
            result = t.matrix() * result;
        }
        return result;
    }
}