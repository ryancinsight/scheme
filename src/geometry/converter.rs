//! src/geometry/converter.rs

use crate::config::ConversionConfig;
use crate::geometry::mod_2d::ChannelSystem;
use crate::geometry::mod_3d::{ChannelSystem3D, Cylinder, Sphere, Volume};

/// Converts a 2D `ChannelSystem` to a 3D `ChannelSystem3D`.
pub fn convert_2d_to_3d(
    system_2d: &ChannelSystem,
    config: &ConversionConfig,
) -> ChannelSystem3D {
    let (box_x, box_y) = system_2d.box_dims;
    let box_z = config.box_z_height;
    let z_center = box_z / 2.0;

    let cylinders = system_2d
        .channels
        .iter()
        .map(|channel| {
            let p1 = system_2d.nodes[channel.from_node].point;
            let p2 = system_2d.nodes[channel.to_node].point;
            Cylinder {
                start: (p1.0, p1.1, z_center), // Horizontal channel at center height
                end: (p2.0, p2.1, z_center),   // Horizontal channel at center height
                radius: channel.width / 2.0,
            }
        })
        .collect();

    let spheres = system_2d
        .nodes
        .iter()
        .map(|node| {
            let max_radius = system_2d
                .channels
                .iter()
                .filter(|c| c.from_node == node.id || c.to_node == node.id)
                .map(|c| c.width / 2.0)
                .fold(0.0, f64::max);

            Sphere {
                center: (node.point.0, node.point.1, z_center),
                radius: max_radius,
            }
        })
        .collect();

    let box_volume = Volume {
        min_corner: (0.0, 0.0, 0.0),
        max_corner: (box_x, box_y, box_z),
    };

    ChannelSystem3D {
        box_volume,
        cylinders,
        spheres,
        cones: Vec::new(),
        tori: Vec::new(),
    }
}