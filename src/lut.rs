use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use glam::DVec3;

#[derive(Debug)]
pub struct Cube3D {
    pub title: String,
    pub size: usize,
    pub vectors: Vec<DVec3>,
}

// Implement the Cube3D struct methods here
impl Cube3D {
    pub fn accessor(&self, r: usize, g: usize, b: usize) -> &DVec3 {
        let idx = r + self.size * g + self.size * self.size * b;
        &self.vectors[idx]
    }
}

pub fn cube3d<P: AsRef<Path>>(path: P) -> io::Result<Cube3D> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();
    
    let mut title = String::new();
    let mut size = 0;
    let mut vectors = Vec::new();
    
    // Skip the first line (comment)
    if let Some(Ok(_)) = lines.next() {
        // Process remaining lines
        while let Some(Ok(line)) = lines.next() {
            let line = line.trim();
            
            if line.starts_with("TITLE") {
                title = line.split_whitespace()
                    .skip(1)  // Skip "TITLE"
                    .collect::<Vec<&str>>()
                    .join(" ");
            } else if line.starts_with("LUT_3D_SIZE") {
                size = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if !line.is_empty() {
                // Try to parse vector values
                let values: Vec<f64> = line.split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                
                if values.len() == 3 {
                    vectors.push(DVec3::new(values[0], values[1], values[2]));
                }
            }
        }
    }
    
    Ok(Cube3D {
        title,
        size,
        vectors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube3d_parsing() {
        let result = cube3d("data/example_short.cube").unwrap();
        assert_eq!(result.title, "SLog3SGamut3.CineToLC_709");
        assert_eq!(result.size, 33);
        assert!(!result.vectors.is_empty());
    }
}
