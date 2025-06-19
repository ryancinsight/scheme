//! src/mesh/stl.rs

use std::fs::File;
use std::io::BufWriter;
use stl_io::Triangle;

/// Writes a mesh to an STL file.
pub fn write_stl(path: &str, triangles: &[Triangle]) -> Result<(), std::io::Error> {
    let mut file = BufWriter::new(File::create(path)?);
    stl_io::write_stl(&mut file, triangles.iter())?;
    println!("STL file saved to {}", path);
    Ok(())
} 