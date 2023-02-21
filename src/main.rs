#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use eframe;
use eframe::glow;
use egui::vec2;
use glm::{Vec3, vec3};
use mesh_widget::*;
mod mesh_widget;
extern crate nalgebra_glm as glm;
mod triangle;

struct AppState {
    gl: Arc<glow::Context>,
    filename: Option<String>,
    triangles: Option<Vec<Triangle>>,
    mesh: Option<ViewState>,
    alert: Option<Arc<Mutex<String>>>
}

fn rotate_triangles(input: &[Triangle]) -> Vec<Triangle> {
    // rotates triangles by -90 degrees around x axis
    let mut result = Vec::<Triangle>::new();
    for triangle in input {
        let mut new_tri = Vec::<Vec3>::new();
        for vertex in triangle {
            new_tri.push(vec3(vertex[0], -vertex[2], vertex[1]));
        }
        result.push([new_tri[0], new_tri[1], new_tri[2]]);
    }
    return result;
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(filename) = &self.filename {
                ui.label(format!("File: {}", filename));
            }
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    if let Some(rfd_result) = rfd::FileDialog::new().add_filter("stl", &["stl", "STL"]).pick_file() {
                        let input_file = rfd_result.display().to_string();
                        self.filename = Some(input_file.clone());
                        self.mesh = match triangle::read_stl_binary(input_file.as_str()) {
                            Err(_) => {
                                None
                            },
                            Ok(mesh) => {
                                let mesh_view_state = ViewState::new(self.gl.to_owned(), &mesh).unwrap();
                                self.triangles = Some(mesh);
                                Some(mesh_view_state)
                            }
                        }
                    }
                }
                if let Some(triangles) = &self.triangles {
                    if ui.button("Export").clicked() {
                        if let Some(rfd_result) = rfd::FileDialog::new().add_filter("stl", &["stl", "STL"]).save_file() {
                            let save_file = rfd_result.display().to_string();
                            match triangle::write_stl_binary(save_file.as_str(), &rotate_triangles(&triangles)) {
                                Err(err) => {
                                    self.show_alert(format!("Could not save mesh:\n\t{}", err));
                                },
                                Ok(_) => {
                                    self.show_alert(format!("Saved: {}", save_file));
                                }
                            }
                        }
                    }
                }
            });
            if let Some(mesh) = &mut self.mesh {
                let view_size = vec2(ui.available_width(), ui.available_height());
                ui.add(mesh_widget::mesh_view(view_size, mesh));
            }
            if let Some(alert) = self.alert.clone() {
                egui::Window::new("Alert")
                    .collapsible(false)
                    .show(ctx, |ui| {
                    let alert = alert.lock().unwrap();
                    ui.vertical_centered(|ui| {
                        ui.spacing();
                        ui.label(alert.as_str());
                        ui.spacing();
                        if ui.button("OK").clicked() {
                            self.alert = None;
                        }
                    })
                });
            }
        });
    }
}

impl AppState {
    fn new(gl: Arc<glow::Context>) -> Self {
        return Self{
            gl: gl,
            filename: None,
            triangles: None,
            mesh: None,
            alert: None
        }
    }
    fn show_alert(&mut self, alert: String) {
        self.alert = Some(Arc::new(Mutex::new(alert)));
    }
}

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::vec2(500., 500.));
    eframe::run_native(
        "Mold 2 Patient",
        options,
        Box::new(|cc|
            Box::new(AppState::new(cc.gl.to_owned().expect("Could not get gl context"))))
    )
}