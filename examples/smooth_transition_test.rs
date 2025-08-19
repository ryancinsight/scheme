use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, WaveShape},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Testing Smooth Transition Envelope");
    println!("=====================================");

    // Create a simple configuration for testing smooth transitions
    let geometry_config = GeometryConfig::default();
    
    let smooth_serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 4.0,
        wave_density_factor: 3.0,
        wave_shape: WaveShape::Sine,
        gaussian_width_factor: 6.0,
        ..SerpentineConfig::default()
    };

    // Create a simple bifurcation to test the smooth transitions
    let system = create_geometry(
        (200.0, 100.0), // 200mm x 100mm box
        &[SplitType::Bifurcation],
        &geometry_config,
        &ChannelTypeConfig::AllSerpentine(smooth_serpentine_config),
    );

    // Save the result
    std::fs::create_dir_all("outputs/smooth_transition_test")?;
    
    plot_geometry(
        &system,
        "outputs/smooth_transition_test/smooth_envelope_test.png",
    )?;

    println!("✅ Smooth transition test complete!");
    println!("   Output saved to: outputs/smooth_transition_test/smooth_envelope_test.png");
    
    // Print some channel information for analysis
    for (i, channel) in system.channels.iter().enumerate() {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            println!("   Channel {}: {} points", i + 1, path.len());
            
            // Check for smooth transitions by analyzing amplitude changes
            let mut max_amplitude_change: f64 = 0.0;
            for j in 1..path.len() {
                let prev_y = path[j - 1].1;
                let curr_y = path[j].1;
                let amplitude_change = (curr_y - prev_y).abs();
                max_amplitude_change = max_amplitude_change.max(amplitude_change);
            }
            println!("   Channel {}: Max amplitude change between points: {:.3}mm", i + 1, max_amplitude_change);
        }
    }

    Ok(())
}
