#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use eframe;
use eframe::glow;
use egui::{Ui, TextureHandle};
use image;
use mesh_widget::*;
mod mesh_widget;
extern crate nalgebra_glm as glm;
mod triangle;

struct AppState {
    gl: Arc<glow::Context>,
    alert: Option<Arc<Mutex<String>>>,
    triangles: Option<Vec<Triangle>>,
    mesh: Option<ViewState>,
    texture: Option<TextureHandle>
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut render_flag = false;
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_mesh_file();
                    }
                    if self.mesh.is_some() {
                        if ui.button("Save").clicked() {
                            self.save_mesh_file_menu(ui);
                        }
                        if ui.button("Save Render").clicked() {
                            render_flag = true;
                        }
                    }
                });
            });
            if let Some(mesh) = &mut self.mesh {
                ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        ui.toggle_value(&mut mesh.right_handed, "right handed");
                        ui.collapsing("Lighting", |ui| {
                            ui.label("Ambient: ");
                            ui.color_edit_button_rgb(&mut mesh.ambient);
                            ui.label("Diffuse: ");
                            ui.color_edit_button_rgb(&mut mesh.diffuse);
                            ui.label("Specular: ");
                            ui.color_edit_button_rgb(&mut mesh.specular);
                            ui.label("Light Source:");
                            let light_dir = -mesh.light_direction.normalize();
                            let mut light_yaw = f32::atan2(light_dir.y, light_dir.x);
                            // If the light is vertical, gimble lock to 0
                            if light_dir.z > 0.999 || light_dir.z < -0.999 {
                                light_yaw = 0.0;
                            }
                            light_yaw = (light_yaw * 180.0 / PI).round() * PI/180.0;
                            let mut light_pitch = light_dir.z.asin();
                            light_pitch = (light_pitch * 180.0 / PI).round() * PI/180.0;
                            ui.drag_angle(&mut light_yaw);
                            ui.drag_angle(&mut light_pitch);
                            light_pitch = light_pitch.clamp(-PI/2., PI/2.);
                            mesh.light_direction = -glm::Vec3::new(light_yaw.cos() * light_pitch.cos(), light_yaw.sin() * light_pitch.cos(), light_pitch.sin());
                        });
                        if ui.button("Screenshot").clicked() {
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                [200,200],
                                &mesh.draw_pixels(200,200).unwrap());
                            self.texture = Some(
                                ui.ctx().load_texture(
                                    "screenshot",
                                    color_image,
                                    Default::default())
                            );
                        }
                        if let Some(texture) = &self.texture {
                            ui.image(texture, texture.size_vec2());
                        }
                    });
                    let size = egui::Vec2::new(ui.available_width(), ui.available_height());
                    if render_flag {
                        self.save_render(
                            &mesh.draw_pixels(size.x as usize, size.y as usize).unwrap(),
                            size.x as usize, size.y as usize);
                    }
                    ui.add(mesh_widget::mesh_view(size, mesh));
                 });
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
            alert: None,
            triangles: None,
            mesh: None,
            texture: None
        }
    }
    fn show_alert(&mut self, alert: String) {
        self.alert = Some(Arc::new(Mutex::new(alert)));
    }
    fn open_mesh_file(&mut self) {
        if let Some(rfd_result) = rfd::FileDialog::new().pick_file() {
            let input_file = rfd_result.display().to_string();
            self.mesh = match triangle::read_stl_binary(input_file.as_str()) {
                Err(_) => {
                    self.show_alert(format!("Could not open file {}", input_file));
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
    fn save_mesh_file_menu(&mut self, ui: &mut Ui) {
        if let Some(triangles) = &self.triangles {
            if let Some(rfd_result) = rfd::FileDialog::new().add_filter("stl", &["stl", "STL"]).save_file() {
                let save_file = rfd_result.display().to_string();
                match triangle::write_stl_binary(save_file.as_str(), &triangles) {
                    Err(err) => {
                        self.show_alert(format!("Could not save mesh:\n\t{}", err));
                    },
                    Ok(_) => {
                        self.show_alert(format!("Saved: {}", save_file));
                    }
                }
            }
        } else {
            self.show_alert("There is no triangle data to save".to_string());
        }
    }
    fn save_render(&mut self, pixels: &Vec<u8>, width: usize, height: usize) {
        if let Some(rfd_result) = rfd::FileDialog::new().add_filter("png", &["png", "PNG"]).save_file() {
            let save_file = rfd_result.display().to_string();
            match image::save_buffer(
                save_file.to_owned(),
                pixels.as_slice(),
                width as u32,
                height as u32,
                image::ColorType::Rgba8) {
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