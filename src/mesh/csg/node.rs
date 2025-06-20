//! src/mesh/csg/node.rs
use super::plane::Plane;
use super::polygon::Polygon;

#[derive(Clone)]
pub struct CsgNode {
    polygons: Vec<Polygon>,
    front: Option<Box<CsgNode>>,
    back: Option<Box<CsgNode>>,
    plane: Option<Plane>,
}

impl CsgNode {
    pub fn new(polygons: Option<Vec<Polygon>>) -> Self {
        let mut node = Self {
            polygons: Vec::new(),
            front: None,
            back: None,
            plane: None,
        };
        if let Some(poly_list) = polygons {
            if !poly_list.is_empty() {
                node.build(poly_list);
            }
        }
        node
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

    pub fn clip_polygons(&self, polygons: &[Polygon]) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons.to_vec();
        }
        let plane = self.plane.as_ref().unwrap();
        let mut all_front_polys = Vec::new();
        let mut all_back_polys = Vec::new();
        for poly in polygons {
            let mut co_planar_front = Vec::new();
            let mut co_planar_back = Vec::new();
            let mut front_polys = Vec::new();
            let mut back_polys = Vec::new();
            plane.split_polygon(
                poly,
                &mut co_planar_front,
                &mut co_planar_back,
                &mut front_polys,
                &mut back_polys,
            );
            all_front_polys.extend(co_planar_front);
            all_front_polys.extend(front_polys);
            all_back_polys.extend(co_planar_back);
            all_back_polys.extend(back_polys);
        }
        if let Some(front_node) = &self.front {
            all_front_polys = front_node.clip_polygons(&all_front_polys);
        }
        if let Some(back_node) = &self.back {
            all_back_polys = back_node.clip_polygons(&all_back_polys);
        } else {
            all_back_polys.clear();
        }
        all_front_polys.extend(all_back_polys);
        all_front_polys
    }

    pub fn clip_to(&mut self, other: &CsgNode) {
        self.polygons = other.clip_polygons(&self.polygons);
        if let Some(front_node) = &mut self.front {
            front_node.clip_to(other);
        }
        if let Some(back_node) = &mut self.back {
            back_node.clip_to(other);
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

    pub fn build(&mut self, polygons: Vec<Polygon>) {
        if self.plane.is_none() {
            self.plane = Some(polygons[0].plane.clone());
        }
        let plane = self.plane.as_ref().unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();
        for poly in polygons {
            let mut co_planar_front = Vec::new();
            let mut co_planar_back = Vec::new();
            let mut temp_front = Vec::new();
            let mut temp_back = Vec::new();
            plane.split_polygon(
                &poly,
                &mut co_planar_front,
                &mut co_planar_back,
                &mut temp_front,
                &mut temp_back,
            );
            self.polygons.extend(co_planar_front);
            self.polygons.extend(co_planar_back);
            front_polys.extend(temp_front);
            back_polys.extend(temp_back);
        }
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
} 