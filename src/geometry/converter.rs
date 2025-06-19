//! src/geometry/converter.rs

use crate::config::ConversionConfig;
use crate::geometry::mod_2d::ChannelSystem;
use crate::geometry::mod_3d::{ChannelSystem3D, Cylinder, Volume};

/// Converts a 2D `ChannelSystem` to a 3D `ChannelSystem3D`.
pub fn convert_2d_to_3d(
    system_2d: &ChannelSystem,
    config: &ConversionConfig,
) -> ChannelSystem3D {
    let (box_x, box_y) = system_2d.box_dims;
    let box_z = config.box_z_height;
    let z_center = box_z / 2.0;

    let box_volume = Volume {
        min_corner: (0.0, 0.0, 0.0),
        max_corner: (box_x, box_y, box_z),
    };

    let cylinders = system_2d
        .channels
        .iter()
        .map(|channel| {
            let p1 = system_2d.nodes[channel.from_node].point;
            let p2 = system_2d.nodes[channel.to_node].point;

            Cylinder {
                start: (p1.0, p1.1, z_center),
                end: (p2.0, p2.1, z_center),
                radius: channel.height / 2.0,
            }
        })
        .collect();

    ChannelSystem3D {
        box_volume,
        cylinders,
    }
} 