//! src/mesh/csg/api.rs
use super::node::CsgNode;
use super::polygon::Polygon;

#[derive(Clone)]
pub struct Csg {
    node: CsgNode,
}

impl Csg {
    pub fn from_polygons(polygons: Vec<Polygon>) -> Self {
        Self {
            node: CsgNode::new(Some(polygons)),
        }
    }

    pub fn to_polygons(&self) -> Vec<Polygon> {
        self.node.all_polygons()
    }

    fn invert(&self) -> Self {
        let mut node = self.node.clone();
        node.invert();
        Self { node }
    }

    pub fn union(&self, other: &Self) -> Self {
        let a_inv = self.invert();
        let b_inv = other.invert();
        let intersect_inverses = a_inv.intersect(&b_inv);
        intersect_inverses.invert()
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let b_inv = other.invert();
        self.intersect(&b_inv)
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let a_polys = self.node.all_polygons();
        let b_polys = other.node.all_polygons();

        let mut a_polys_inside_b = other.node.clip_polygons(a_polys);
        let mut b_polys_inside_a = self.node.clip_polygons(b_polys);

        a_polys_inside_b.append(&mut b_polys_inside_a);
        Self::from_polygons(a_polys_inside_b)
    }

    pub fn xor(&self, other: &Self) -> Self {
        let union_ab = self.union(other);
        let intersect_ab = self.intersect(other);
        union_ab.subtract(&intersect_ab)
    }
} 