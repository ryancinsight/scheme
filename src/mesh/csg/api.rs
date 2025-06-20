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

    pub fn union(&self, other: &Self) -> Self {
        // Union: A ∪ B - Combines both objects
        let mut a = self.node.clone();
        let mut b = other.node.clone();
        
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(b.all_polygons());
        
        Self { node: a }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        // Subtract: A - B - Removes B from A
        // This was previously labeled as "intersect" but is actually subtract
        let mut a = self.node.clone();
        let mut b = other.node.clone();
        
        a.invert();
        b.clip_to(&a);
        b.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        a.build(b.all_polygons());
        a.invert();
        
        Self { node: a }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        // Intersect: A ∩ B - Shows only the overlapping volume
        // This was previously labeled as "subtract" but is actually intersect
        let mut a = self.node.clone();
        let mut b = other.node.clone();
        
        a.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(b.all_polygons());
        a.invert();
        
        Self { node: a }
    }

    pub fn xor(&self, other: &Self) -> Self {
        // XOR: A ⊕ B = (A ∪ B) - (A ∩ B) - Symmetric difference
        let union_ab = self.union(other);
        let intersect_ab = self.intersect(other);
        union_ab.subtract(&intersect_ab)
    }
} 