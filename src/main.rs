#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use eframe;
use eframe::glow;
use mesh_view::*;
mod mesh_view;
mod triangle;

struct AppState {
    gl: Arc<glow::Context>,
    mesh: Option<Arc<Mutex<RenderableMesh>>>
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_mesh_file();
                    }
                });
            });
            if let Some(mesh) = &mut self.mesh {
                ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        let mut mesh = mesh.lock().unwrap();
                        ui.toggle_value(&mut mesh.right_handed, "right handed");
                        ui.collapsing("Lighting", |ui| {
                            ui.label("Ambient: ");
                            ui.color_edit_button_rgb(&mut mesh.ambient);
                            ui.label("Diffuse: ");
                            ui.color_edit_button_rgb(&mut mesh.diffuse);
                            ui.label("Specular: ");
                            ui.color_edit_button_rgb(&mut mesh.specular);
                        });
                    });
                    let max_size = f32::max(ui.available_height(), ui.available_width());
                    let size = egui::Vec2::new(max_size, max_size);
                    ui.add(MeshView::new(size, mesh.to_owned()));
                });
            }
        });
    }
}

impl AppState {
    fn new(gl: Arc<glow::Context>) -> Self {
        return Self{
            gl: gl,
            mesh: None
        }
    }
    fn open_mesh_file(&mut self) {
        if let Some(rfd_result) = rfd::FileDialog::new().pick_file() {
            let input_file = rfd_result.display().to_string();
            self.mesh = match triangle::read_stl_binary(input_file.as_str()) {
                Err(_) => {None},
                Ok(mesh) => {
                    let renderable_mesh = RenderableMesh::new(self.gl.to_owned(), mesh).unwrap();
                    Some(Arc::new(Mutex::new(renderable_mesh)))
                }
            }
        }
    }
}

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::vec2(800., 600.));
    eframe::run_native(
        "Mesh Tools",
        options,
        Box::new(|cc|
            Box::new(AppState::new(cc.gl.to_owned().expect("Could not get gl context"))))
    )
}