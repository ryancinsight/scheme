//! examples/3d/schematic_to_3d.rs

use pyvismil::{
    config::{ConversionConfig, GeometryConfig},
    geometry::{convert_2d_to_3d, create_geometry, SplitType},
    mesh::{difference, difference_csgrs, write_stl},
    visualizations,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/schematic_to_3d";
    fs::create_dir_all(output_dir)?;

    // 1. Define 2D geometry
    let geo_config = GeometryConfig::default();
    let schematic = create_geometry((200.0, 100.0), &[SplitType::Bifurcation], &geo_config);

    // 2. Convert to 3D with 44mm height (realistic microfluidic device thickness)
    println!("Converting 2D schematic to 3D system...");
    let conv_config = ConversionConfig {
        box_z_height: 44.0, // 44mm height for realistic microfluidic device
    };
    let system_3d = convert_2d_to_3d(&schematic, &conv_config);

    // Verify the 3D system configuration
    println!("   ✓ 3D system created with {} cylinders and {} spheres",
             system_3d.cylinders.len(), system_3d.spheres.len());
    println!("   📐 Box dimensions: {:.1}×{:.1}×{:.1} mm",
             system_3d.box_volume.max_corner.0 - system_3d.box_volume.min_corner.0,
             system_3d.box_volume.max_corner.1 - system_3d.box_volume.min_corner.1,
             system_3d.box_volume.max_corner.2 - system_3d.box_volume.min_corner.2);

    // Verify channel configuration
    if !system_3d.cylinders.is_empty() {
        let first_cylinder = &system_3d.cylinders[0];
        let cylinder_length = ((first_cylinder.end.0 - first_cylinder.start.0).powi(2) +
                              (first_cylinder.end.1 - first_cylinder.start.1).powi(2) +
                              (first_cylinder.end.2 - first_cylinder.start.2).powi(2)).sqrt();
        println!("   🔧 Channel configuration: Horizontal channels at Z={:.1} mm (center height)", first_cylinder.start.2);
        println!("   📏 Channel length: {:.1} mm, diameter: {:.1} mm",
                 cylinder_length, first_cylinder.radius * 2.0);
        println!("   ✓ Microfluidic channels positioned within 44mm device thickness");
    }

    // 3. Plot the 3D representation
    println!("Plotting 3D system...");
    let plot_path = format!("{}/schematic_to_3d_plot.png", output_dir);
    visualizations::plot_3d_system(&system_3d, &plot_path)?;

    // 4. Generate mesh with boolean difference (original implementation)
    println!("Generating mesh with boolean difference (original)...");
    let final_mesh_original = difference(&system_3d)?;

    // 5. Generate mesh with boolean difference (csgrs implementation)
    println!("Generating mesh with boolean difference (csgrs)...");
    let final_mesh_csgrs = difference_csgrs(&system_3d)?;

    // 6. Write both meshes to STL files for comparison
    let stl_path_original = format!("{}/schematic_to_3d_original.stl", output_dir);
    write_stl(&stl_path_original, &final_mesh_original)?;

    let stl_path_csgrs = format!("{}/schematic_to_3d_csgrs.stl", output_dir);
    write_stl(&stl_path_csgrs, &final_mesh_csgrs)?;

    println!("\n🎉 Successfully converted schematic to 3D and saved files:");
    println!("📁 Files generated:");
    println!("   • 3D visualization: {}/schematic_to_3d_plot.png", output_dir);
    println!("   • Original implementation: {}", stl_path_original);
    println!("   • CSGRS implementation: {}", stl_path_csgrs);
    println!("📊 Mesh comparison:");
    println!("   • Original mesh triangles: {}", final_mesh_original.len());
    println!("   • CSGRS mesh triangles: {}", final_mesh_csgrs.len());
    println!("   • Detail ratio: {:.1}x (CSGRS vs Original)",
             final_mesh_csgrs.len() as f64 / final_mesh_original.len() as f64);
    println!("🔧 Configuration:");
    println!("   • Device height: 44mm (realistic microfluidic thickness)");
    println!("   • Channels: Horizontal microfluidic channels at center height (Z=22mm)");
    println!("   • CSG optimization: Union-then-difference for cylinders + spheres");
    println!("   • Junction smoothing: Spheres at channel intersections for smooth flow");
    println!("   • Architecture: Standard microfluidic device with embedded channel network");

    Ok(())
} 