use std::sync::Arc;

use crate::thread_request::Request;
use crate::triangle;
use crate::triangle::Triangle;
use super::mesh::TriangleMesh;


pub struct AnalysisUI {
    original_face_count: usize,
    mesh: Option<Request<TriangleMesh>>,
    surface_area: Option<Request<f32>>,
    volume: Option<Request<f32>>,
    closed: Option<Request<bool>>,
    body_count: Option<Request<usize>>,
    holes: Option<Request<usize>>
}

impl AnalysisUI {
    pub fn new(triangles: Option<&[Triangle]>) -> Self {
        let original_face_count = match triangles {
            None => 0,
            Some(t) => t.len()
        };
        let (mesh, surface_area, volume) = match triangles {
            None => (None, None, None),
            Some(t) => {
                let owned_triangles = Arc::new(t.to_owned());
                let triangles = owned_triangles.clone();
                let mesh = Some(Request::new(move || {
                    return TriangleMesh::new(&triangles);
                }));
                let triangles = owned_triangles.clone();
                let surface_area = Some(Request::new(move || {
                    return triangle::surface_area(&triangles);
                }));
                let triangles = owned_triangles.clone();
                let volume = Some(Request::new(move || {
                    return triangle::volume(&triangles);
                }));
                (mesh, surface_area, volume)
            }
        };
        Self {
            original_face_count,
            mesh,
            surface_area,
            volume,
            closed: None,
            body_count: None,
            holes: None
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Surface Area: ");
            if let Some(surface_area) = &self.surface_area {
                if let Some(surface_area) = &*surface_area.result().read().unwrap() {
                    ui.label(format!("{}", surface_area));
                } else {
                    ui.spinner();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Volume: ");
            if let Some(volume) = &self.volume {
                if let Some(volume) = &*volume.result().read().unwrap() {
                    ui.label(format!("{}", volume));
                } else {
                    ui.spinner();
                }
            }
        });
        if let Some(mesh_request) = &self.mesh {
            let mesh = mesh_request.result().clone();
            if let Some(mesh) = &*mesh.read().unwrap() {
                ui.horizontal(|ui| {
                    ui.label("Body Count: ");
                    if let Some(body_count) = &self.body_count {
                        if let Some(res) = &*body_count.result().read().unwrap() {
                            ui.label(format!("{}", res));
                        } else {
                            ui.spinner();
                        }
                    } else {
                        // Create a copy to send to the body count thread
                        let mesh_result = mesh_request.result().clone();
                        if ui.button("Compute Body Count").clicked() {
                            self.body_count = Some(Request::new(move || {
                                if let Some(mesh) = &*mesh_result.read().unwrap() {
                                    return mesh.count_bodies();
                                } else {
                                    return 0;
                                }
                            }))
                        }
                    }
                });
            };
        }
    }
}