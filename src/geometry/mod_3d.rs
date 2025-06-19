//! geometry/mod_3d.rs

pub type Point3D = (f64, f64, f64);

/// Represents a 3D cuboid volume defined by two opposing corners.
#[derive(Debug, Clone)]
pub struct Volume {
    pub min_corner: Point3D,
    pub max_corner: Point3D,
}

impl Volume {
    /// Returns the 8 vertices of the volume.
    pub fn get_vertices(&self) -> [Point3D; 8] {
        [
            (self.min_corner.0, self.min_corner.1, self.min_corner.2),
            (self.max_corner.0, self.min_corner.1, self.min_corner.2),
            (self.max_corner.0, self.max_corner.1, self.min_corner.2),
            (self.min_corner.0, self.max_corner.1, self.min_corner.2),
            (self.min_corner.0, self.min_corner.1, self.max_corner.2),
            (self.max_corner.0, self.min_corner.1, self.max_corner.2),
            (self.max_corner.0, self.max_corner.1, self.max_corner.2),
            (self.min_corner.0, self.max_corner.1, self.max_corner.2),
        ]
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            min_corner: (0.0, 0.0, 0.0),
            max_corner: (0.0, 0.0, 0.0),
        }
    }
}

/// Represents a cylinder defined by its start point, end point, and radius.
#[derive(Debug, Clone)]
pub struct Cylinder {
    pub start: Point3D,
    pub end: Point3D,
    pub radius: f64,
}

/// Represents a sphere defined by its center point and radius.
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point3D,
    pub radius: f64,
}

/// Represents the entire 3D channel system.
#[derive(Debug, Clone)]
pub struct ChannelSystem3D {
    pub box_volume: Volume,
    pub cylinders: Vec<Cylinder>,
    pub spheres: Vec<Sphere>,
}

impl ChannelSystem3D {
    pub fn has_drawable_box(&self) -> bool {
        self.box_volume.min_corner != self.box_volume.max_corner
    }

    pub fn get_bounding_box(&self) -> (Point3D, Point3D) {
        let mut points_to_bound: Vec<Point3D> = Vec::new();

        if self.has_drawable_box() {
            points_to_bound.push(self.box_volume.min_corner);
            points_to_bound.push(self.box_volume.max_corner);
        }

        for cyl in &self.cylinders {
            points_to_bound.push((
                cyl.start.0.min(cyl.end.0) - cyl.radius,
                cyl.start.1.min(cyl.end.1) - cyl.radius,
                cyl.start.2.min(cyl.end.2) - cyl.radius,
            ));
            points_to_bound.push((
                cyl.start.0.max(cyl.end.0) + cyl.radius,
                cyl.start.1.max(cyl.end.1) + cyl.radius,
                cyl.start.2.max(cyl.end.2) + cyl.radius,
            ));
        }

        for sphere in &self.spheres {
            points_to_bound.push((
                sphere.center.0 - sphere.radius,
                sphere.center.1 - sphere.radius,
                sphere.center.2 - sphere.radius,
            ));
            points_to_bound.push((
                sphere.center.0 + sphere.radius,
                sphere.center.1 + sphere.radius,
                sphere.center.2 + sphere.radius,
            ));
        }

        if points_to_bound.is_empty() {
            return ((-1.0, -1.0, -1.0), (1.0, 1.0, 1.0)); // Default bounding box
        }

        let mut min_p = points_to_bound[0];
        let mut max_p = points_to_bound[0];

        for p in points_to_bound.iter().skip(1) {
            min_p.0 = min_p.0.min(p.0);
            min_p.1 = min_p.1.min(p.1);
            min_p.2 = min_p.2.min(p.2);
            max_p.0 = max_p.0.max(p.0);
            max_p.1 = max_p.1.max(p.1);
            max_p.2 = max_p.2.max(p.2);
        }

        (min_p, max_p)
    }
} 