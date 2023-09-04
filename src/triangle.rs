use std::fs::File;
use std::io::{Write, Read, BufReader};
extern crate nalgebra_glm as glm;
use glm::{Vec3, Mat4};

pub type Triangle = [Vec3; 3];

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
    triangles: &[Triangle])
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

/// Writes triangles to a binary stl file.
/// The normal is set based on the triangle vertices.
/// Gives no data (0x00...) for header and attributes.
pub fn write_transformed_stl_binary(
    path: &str,
    triangles: &[Triangle],
    transformation: &Mat4)
-> Result<(), std::io::Error> {
    let transformed_triangles: Vec<Triangle> =
        triangles.iter()
        .map(|t| {transform(t, &transformation)})
        .collect();
    return write_stl_binary(path, &transformed_triangles);
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