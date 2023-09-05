/// UNUSED

use glm::max;
use glm::min;
use glm::Vec3;
use glm::TVec3;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::vec::*;

use crate::triangle;
use crate::triangle::Triangle;

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    /// List of vertices
    vertices: Vec<Vec3>,
    /// List of faces, represented by three vertex indexes
    faces: Vec<TVec3<usize>>,
    /// Map of faces pointed to by each vertex
    face_map: Vec<Vec<usize>>
}

fn x_less(a: &Vec3, b: &Vec3) -> bool{
    return a.x < b.x;
}

// Returns the index *not* less than the given value in O(log(n)) time.
// If all values are less than the given value, returns the list size.
fn binary_min_search<T>(list: &[T], value: &T, less: fn(&T,&T)->bool) -> usize {
    if list.len() ==0 {
        return 0;
    }
    let index = list.len() / 2;
    if less(&list[index], &value) {
        return index + 1 + binary_min_search(&list[index + 1..], value, less);
    } else {
        return binary_min_search(&list[0..index], value, less);
    }
}

pub fn merge_vertices<T: Iterator<Item = Vec3>>(vertices: T, min_axis_distance: f32)
-> HashMap::<TVec3<i64>, (Vec3, Vec<usize>)> {
    // make a quantized map to help merge vertices quickly.
    let mut quant_map = HashMap::<TVec3<i64>, (Vec3, Vec<usize>)>::new();
    let mut i = 0;
    for v in vertices {
        let quant = TVec3::new(
            (v[0] / min_axis_distance).floor() as i64,
            (v[1] / min_axis_distance).floor() as i64,
            (v[2] / min_axis_distance).floor() as i64);
        let mut unique = true;
        for x in quant.x - 1 .. quant.x + 1 {
            for y in quant.y - 1 .. quant.y + 1 {
                for z in quant.z - 1 .. quant.z + 1 {
                    let check_quant = TVec3::new(x,y,z);
                    if let Some((existing_vertex, indexes)) = quant_map.get_mut(&check_quant) {
                        if check_quant == quant || (*existing_vertex - v).max() < min_axis_distance {
                            unique = false;
                            indexes.push(i);
                            break;
                        }
                    }
                }
                if !unique {break};
            }
            if !unique {break};
        }
        if unique {
            quant_map.insert(quant, (v, vec![i]));
        }
        i += 1;
    }
    return quant_map;
}

impl TriangleMesh {
    /// Gets the bounding box of a mesh as (minimum_corner, maximum_corner)
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut min_corner = self.vertices.get(0)?.to_owned();
        let mut max_corner = min_corner.clone();
        for v in &self.vertices {
            for i in 0..3 {
                max_corner[i] = max_corner[i].max(v[i]);
                min_corner[i] = min_corner[i].min(v[i]);
            }
        }
        return Some((min_corner, max_corner));
    }

    pub fn new(triangles: &[Triangle]) -> Self {
        let (min, max) = match triangle::bounding_box(triangles) {
            Some(x) => {x},
            None => {(Vec3::zeros(), Vec3::zeros())}
        };
        // An f32 has 23 bits of mantissa.
        // This will ignore error outside of 16 bits of the largest magnitude value
        let tolerance = (max - min).max() / 65536.0;
        // This will remove zero-area triangles.
        let nonzero_triangles = triangles.iter().filter(|t| {
            return (t[0] - t[1]).abs().max() >= tolerance &&
                (t[1] - t[2]).abs().max() >= tolerance &&
                (t[2] - t[0]).abs().max() >= tolerance
        });
        let vertex_map = merge_vertices(nonzero_triangles.flatten().map(|x| x.to_owned()), tolerance);
        let mut vertices = Vec::<Vec3>::new();
        let mut faces = vec![TVec3::<usize>::new(0,0,0); triangles.len()];
        let mut face_map = Vec::<Vec::<usize>>::new();
        for (_, (vertex, vertex_map)) in vertex_map{
            for i in &vertex_map {
                faces[i/3][i%3] = vertices.len();
            }
            vertices.push(vertex);
            face_map.push(vertex_map.iter().map(|x| x/3).collect());
        }
        return Self { vertices, faces, face_map };
    }

    pub fn count_bodies(&self) -> usize {
        // Mark the island of each vertex (0 representing no island)
        let mut island_markers = vec![0; self.vertices.len()];
        let mut island_count = 0;
        for i in 0..self.vertices.len() {
            if island_markers[i] != 0 {
                // This vertex has already been added to an island
                continue;
            }
            // mark island and propogate
            island_count += 1;
            island_markers[i] = island_count;
            let mut open_set = vec![i];
            while open_set.len() > 0 {
                let vert = open_set.pop().unwrap();
                for face in &self.face_map[vert] {
                    for adjacent_vert in &self.faces[*face] {
                        if island_markers[*adjacent_vert] == 0 {
                            open_set.push(*adjacent_vert);
                            island_markers[*adjacent_vert] = island_count;
                        }
                    }
                }
            }
        }
        return island_count;
    }

    // fn cleanup(&mut self) {
    //     // Use the bounds to determine the tolerance
    //     let (min_corner, max_corner) = self.bounds();
    //     let mesh_size = max_corner - min_corner;
    //     let largest_max = mesh_size[0].max(mesh_size[1]).max(mesh_size[2]);
    //     let largest_min = -mesh_size[0].min(mesh_size[1]).min(mesh_size[2]);
    //     // An f32 has 23 bits of mantissa.
    //     // This will ignore error outside of 16 bits of the largest magnitude value
    //     let tolerance = max(largest_max, largest_min) / 65536 as f32;

    //     let mut sorted_vertices: Vec::<usize, Vec3> = self.vertices.iter().enumerate().collect();
    //     sorted_vertices.sort_unstable_by(|a, b| a.1.x.partial_cmp(&b.1.x).expect("Tried to clean up a mesh with a NaN value"));

    //     let mut index_map = vec![0 as usize; self.vertices.len()];
    //     let mut new_vertices = Vec::<Vec3>::new();
    //     for v in &sorted_vertices {
    //         if new_vertices.len() == 0 {
    //             new_vertices.push(v.to_owned());
    //             continue;
    //         }
    //         let mut i = new_vertices.len() - 1;
    //         // Search backwards for duplicate vertices until the x axis is out of range
    //         while (v.x - tolerance) < new_vertices[i].x {
    //             if glm::distance(&new_vertices[i], *v) < tolerance {
    //                 new_vertices.push(v.to_owned());
    //             }
    //             // If you've reached the beginning, there's nowhere left to search
    //             // This would underflow and break the loop anyways, but handling
    //             // it explicitly seems cleaner.
    //             if i == 0 {
    //                 break;
    //             }
    //             i -= 1;
    //         }
    //     }
    //     //let mut new_faces: Vec::<Vec3>::new();
    // }
}

fn read_vec3(buffer: &mut BufReader<File>, read: fn([u8; 4]) -> f32) -> Result<Vec3, std::io::Error> {
    let mut bytes = [0u8; 4];
    buffer.read_exact(&mut bytes)?;
    let x = read(bytes);
    buffer.read_exact(&mut bytes)?;
    let y = read(bytes);
    buffer.read_exact(&mut bytes)?;
    let z = read(bytes);
    return Ok(Vec3::new(x, y, z));
}

// /// Loads a binary STL file into a header, triangle mesh, list of normals, and list of attributes
// fn load_binary_stl(path: &str) -> Result<([u8; 80], TriangleMesh, Vec<Vec3>, Vec<u16>), std::io::Error> {
//     let mut header = [0u8; 80];
//     let mut triangle_count : u32 = 0;
//     let mut vertices = Vec::<Vec3>::new();
//     let mut faces = Vec::<Vector3::<u32>>::new();
//     let mut attributes = Vec::<u16>::new();
//     let mut normals = Vec::<Vec3>::new();
//     let input_file = File::open(path)?;
//     let mut buffer = BufReader::new(input_file);
//     buffer.read_exact(&mut header)?;
//     let mut bytes = [0u8; 4];
//     buffer.read_exact(&mut bytes)?;
//     triangle_count = u32::from_le_bytes(bytes);
//     let mut attribute_bytes = [0u8; 2];
//     for _i in [0..triangle_count] {
//         faces.push(Vector3::<u32>{x: vertices.len() as u32, y: vertices.len() as u32 + 1, z: vertices.len() as u32 + 2});
//         normals.push(read_vec3(&mut buffer, f32::from_le_bytes)?);
//         vertices.push(read_vec3(&mut buffer, f32::from_le_bytes)?);
//         vertices.push(read_vec3(&mut buffer, f32::from_le_bytes)?);
//         vertices.push(read_vec3(&mut buffer, f32::from_le_bytes)?);
//         buffer.read_exact(&mut attribute_bytes)?;
//         attributes.push(u16::from_le_bytes(attribute_bytes));
//     }
//     return Ok((header, TriangleMesh {vertices: vertices, faces: faces}, normals, attributes));
// }

fn test_binary_min_search() {
    let list = vec![
        Vec3::new(0.,0.,0.),
        Vec3::new(1.,1.,1.),
        Vec3::new(1.,1.,1.),
        Vec3::new(3.,3.,3.),
        Vec3::new(4.,4.,4.)];
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(-1.,0.,0.), x_less), 0);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(0.,0.,0.), x_less), 0);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(1.,0.,0.), x_less), 1);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(2.,0.,0.), x_less), 3);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(3.,0.,0.), x_less), 3);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(4.,0.,0.), x_less), 4);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(5.,0.,0.), x_less), 5);
    assert_eq!(binary_min_search(list.as_slice(), &Vec3::new(6.,0.,0.), x_less), 5);
}

fn main() {
    test_binary_min_search();
    let triangles = triangle::read_stl_binary("/home/kenkron/2cubes.stl").unwrap();
    let mesh = TriangleMesh::new(&triangles);
    println!("{}", mesh.vertices.len());
}
