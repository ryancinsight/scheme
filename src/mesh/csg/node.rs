//! src/mesh/csg/node.rs
use super::plane::Plane;
use super::polygon::Polygon;

#[derive(Clone)]
pub struct CsgNode {
    plane: Option<Plane>,
    polygons: Vec<Polygon>,
    front: Option<Box<CsgNode>>,
    back: Option<Box<CsgNode>>,
}

impl CsgNode {
    pub fn new(polygons: Option<Vec<Polygon>>) -> Self {
        let mut node = Self {
            plane: None,
            polygons: Vec::new(),
            front: None,
            back: None,
        };
        if let Some(poly_list) = polygons {
            node.build(poly_list);
        }
        node
    }

    pub fn build(&mut self, polygons: Vec<Polygon>) {
        if polygons.is_empty() {
            return;
        }

        if self.plane.is_none() {
            // Take the first polygon's plane as the node's splitting plane.
            self.plane = Some(polygons[0].plane.clone());
        }

        let plane = self.plane.as_ref().unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();
        let mut co_planar_front = Vec::new();
        let mut co_planar_back = Vec::new();

        for poly in polygons {
            plane.split_polygon(&poly, &mut co_planar_front, &mut co_planar_back, &mut front_polys, &mut back_polys);
        }
        
        self.polygons = co_planar_front;
        // Note: co_planar_back is discarded as per standard CSG algorithm for building.

        if !front_polys.is_empty() {
            if self.front.is_none() {
                self.front = Some(Box::new(CsgNode::new(None)));
            }
            self.front.as_mut().unwrap().build(front_polys);
        }

        if !back_polys.is_empty() {
            if self.back.is_none() {
                self.back = Some(Box::new(CsgNode::new(None)));
            }
            self.back.as_mut().unwrap().build(back_polys);
        }
    }
    
    pub fn all_polygons(&self) -> Vec<Polygon> {
        let mut polys = self.polygons.clone();
        if let Some(front_node) = &self.front {
            polys.extend(front_node.all_polygons());
        }
        if let Some(back_node) = &self.back {
            polys.extend(back_node.all_polygons());
        }
        polys
    }

    pub fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons;
        }

        let plane = self.plane.as_ref().unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();
        let mut co_planar_front = Vec::new();
        let mut co_planar_back = Vec::new();

        for poly in &polygons {
            plane.split_polygon(poly, &mut co_planar_front, &mut co_planar_back, &mut front_polys, &mut back_polys);
        }

        if let Some(front_node) = &self.front {
            front_polys = front_node.clip_polygons(front_polys);
        }
        if let Some(back_node) = &self.back {
            back_polys = back_node.clip_polygons(back_polys);
        } else {
            back_polys.clear();
        }

        front_polys.append(&mut back_polys);
        front_polys
    }

    pub fn invert(&mut self) {
        for poly in &mut self.polygons {
            poly.flip();
        }
        if let Some(plane) = &mut self.plane {
            plane.flip();
        }
        if let Some(front_node) = &mut self.front {
            front_node.invert();
        }
        if let Some(back_node) = &mut self.back {
            back_node.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }
} 