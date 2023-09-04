use std::fs::File;
use std::io::{Write, Read, BufReader};
extern crate nalgebra_glm as glm;
use glm::Vec3;

pub type Triangle = [Vec3; 3];

pub fn area(triangle: &Triangle) -> f32 {
    let sides = [triangle[1] - triangle[0], triangle[2] - triangle[0]];
    return sides[0].cross(&sides[1]).magnitude() * 0.5;
}

pub fn transform(triangle: &Triangle, transformation: &Mat4) -> Triangle {
    return triangle.map(|vector| transformation.transform_vector(&vector));
}

fn write_vec3(file: &mut File, vector: &Vec3)
-> Result<(), std::io::Error>{
    file.write_all(&vector[0].to_le_bytes())?;
    file.write_all(&vector[1].to_le_bytes())?;
    file.write_all(&vector[2].to_le_bytes())?;
    return Ok(());
}

/// Writes triangles to a binary stl file.
/// The normal is set based on the triangle vertices.
/// Gives no data (0x00...) for header and attributes.
pub fn write_stl_binary(
    path: &str,
    triangles: &Vec::<Triangle>)
-> Result<(), std::io::Error> {
    let mut output = File::create(path)?;
    output.write_all(&[0 as u8; 80])?;
    output.write_all(&(triangles.len() as u32).to_le_bytes())?;
    for triangle in triangles {
        let edge1 = triangle[1] - triangle[0];
        let edge2 = triangle[2] - triangle[0];
        let normal = glm::cross(&edge1, &edge2).normalize();
        write_vec3(&mut output, &normal)?;
        for vertex in triangle {
            write_vec3(&mut output, vertex)?;
        }
        output.write(&[0 as u8; 2])?;
    }
    return Ok(());
}

fn read_vec3(buffer: &mut BufReader<File>) -> Result<Vec3, std::io::Error> {
    let mut bytes = [0u8; 4];
    buffer.read_exact(&mut bytes)?;
    let x = f32::from_le_bytes(bytes);
    buffer.read_exact(&mut bytes)?;
    let y = f32::from_le_bytes(bytes);
    buffer.read_exact(&mut bytes)?;
    let z = f32::from_le_bytes(bytes);
    return Ok(Vec3::new(x, y, z));
}

/// Loads a binary STL file into a list of triangles
///
/// Discards header, normals, and attributes
pub fn read_stl_binary(path: &str) -> Result<Vec::<Triangle>, std::io::Error> {
    let mut header = [0u8; 80];
    let mut triangles = Vec::<Triangle>::new();
    let mut input = BufReader::new(File::open(path)?);
    input.read_exact(&mut header)?;
    let mut bytes = [0u8; 4];
    input.read_exact(&mut bytes)?;
    let triangle_count = u32::from_le_bytes(bytes);
    let mut attribute_bytes = [0u8; 2];
    for _i in 0..triangle_count {
        let _normal = read_vec3(&mut input)?;
        triangles.push([
            read_vec3(&mut input)?,
            read_vec3(&mut input)?,
            read_vec3(&mut input)?]);
        input.read_exact(&mut attribute_bytes)?;
    }
    return Ok(triangles);
}

/// Returns the bounding box of a list of triangles, or None if there are no triangles
pub fn bounding_box(triangles: &[Triangle]) -> Option<(Vec3, Vec3)> {
    if triangles.len() == 0 {
        return None;
    }
    let (mut min, mut max) = (triangles[0][0], triangles[0][0]);
    for t in triangles {
        for v in t {
            for i in 0..v.len() {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
        }
    }
    return Some((min, max));
}

/// Returns the total surface area of a list of triangles
pub fn surface_area(triangles: &[Triangle]) -> f32 {
    return triangles.iter()
        .map(|t| area(t))
        .fold(0.0, |acc, val| acc + val);
}

pub fn volume(triangles: &[Triangle]) -> f32 {
    let origin = Vec3::zeros();
    return triangles.iter()
        .map(|triangle| {
            let sides = [triangle[1] - triangle[0], triangle[2] - triangle[0]];
            let cross = sides[0].cross(&sides[1]);
            let area = cross.magnitude() * 0.5;
            let height = cross.normalize().dot(&(triangle[0] - origin));
            return area * height / 3.0;
        })
        .fold(0.0, |acc, val| acc + val);
}