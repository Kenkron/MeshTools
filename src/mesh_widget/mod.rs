use std::sync::Arc;

use eframe::egui_glow;
extern crate nalgebra_glm as glm;
use glm::Vec3;

pub type Triangle = [Vec3; 3];

mod glow_state;
mod view_state;
pub use glow_state::GlowState;
pub use view_state::ViewState;

/// A simple Widget to view Triangles in 3D space
///
/// Primary mouse drag rotates the model
/// Secondary mouse drag translates the model
/// Middle mouse drag scales the model
fn mesh_ui(ui: &mut egui::Ui, view_size: egui::Vec2, state: &mut ViewState)
-> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(view_size, egui::Sense::drag());

    // Avoids division by zero for translation (and saves a bit of processing)
    if view_size.x * view_size.y == 0. {
        return response;
    }

    let aspect_ratio = view_size.x/view_size.y;
    {

        if response.dragged_by(egui::PointerButton::Primary) {
            state.rotate_y(-response.drag_delta().x * 0.01);
            state.rotate_x(-response.drag_delta().y * 0.01);
        }
        if response.dragged_by(egui::PointerButton::Secondary) {
            let matrix = state.combine_transformations();
            if let Some(inverse_matrix) = matrix.try_inverse() {
                let delta4 = inverse_matrix * glm::Vec4::new(
                    aspect_ratio * 2. * response.drag_delta().x / view_size.x,
                    -2. * response.drag_delta().y / view_size.y,
                    0., 0.);
                state.translation += Vec3::new(delta4.x, delta4.y, delta4.z);
            }
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            state.scale *= std::f32::consts::E.powf(-response.drag_delta().y * 0.01);
        }
    }

    let cb = egui_glow::CallbackFn::new(move |_info, _painter| {
        state.draw(aspect_ratio);
    });

    if ui.is_rect_visible(rect) {
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        });
    }
    return response;
}

pub fn mesh_view(view_size: egui::Vec2, state: &mut ViewState) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| mesh_ui(ui, view_size, state)
}