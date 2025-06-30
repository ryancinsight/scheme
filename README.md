# Scheme - 2D Microfluidic Schematic Design Library

A focused Rust library for designing 2D microfluidic schematics with support for bifurcation and trifurcation patterns, channel layout algorithms, and schematic visualization.

## Features

- **Bifurcation Patterns**: Generate single to five-level bifurcation channel layouts
- **Trifurcation Patterns**: Create single to five-level trifurcation designs  
- **Mixed Patterns**: Combine bifurcation and trifurcation in complex layouts
- **2D Visualization**: Export schematics as PNG images using plotters
- **Configurable Geometry**: Customize channel dimensions and wall clearances
- **Channel Types**: Support for straight and serpentine (S-shaped) channels

## Channel Types

Scheme supports different channel geometries as part of its DAG architecture with precise endpoint alignment:

- **Straight Channels**: Traditional linear connections between nodes
- **Serpentine Channels**: S-shaped channels with configurable amplitude, wavelength, and periods

### Serpentine Channel Features

- **Precise Endpoint Alignment**: Serpentine channels start and end exactly at node positions
- **Gaussian Tone Burst Envelope**: Uses a Gaussian-shaped amplitude envelope for natural transitions
- **Linear Near Nodes**: Channels remain linear near connection points, becoming serpentine in the middle
- **Smooth Channel Separation**: Allows channels to separate before becoming serpentine
- **Dynamic Wavelength Adaptation**: Wavelength automatically adapts to channel length for optimal appearance
- **Configurable Parameters**: Control amplitude factor, wavelength factor, periods, and Gaussian width

Channel types can be configured using the `ChannelTypeConfig` enum:

```rust
use scheme::config::{ChannelTypeConfig, SerpentineConfig};

// All straight channels
let config = ChannelTypeConfig::AllStraight;

// All serpentine channels with custom parameters
let config = ChannelTypeConfig::AllSerpentine(SerpentineConfig {
    fill_factor: 0.8,           // Fraction of available vertical space to fill (0.0 to 1.0)
    wavelength_factor: 4.0,     // Multiplier for channel width to determine wavelength
    gaussian_width_factor: 6.0, // Controls width of Gaussian envelope (sigma = length / gaussian_width_factor)
    wave_density_factor: 1.5,   // Controls how many waves appear relative to channel length (higher = more waves)
});

// Mixed channels based on position (default)
let config = ChannelTypeConfig::MixedByPosition {
    middle_zone_fraction: 0.4,  // Fraction of box width for serpentine zone
    serpentine_config: SerpentineConfig::default(),
};

// Custom channel type function with length-based selection
let config = ChannelTypeConfig::Custom(|from, to, box_dims| {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let length = (dx * dx + dy * dy).sqrt();
    
    if length > 8.0 {
        ChannelType::Serpentine {
            amplitude: length * 0.15,   // Scale amplitude with length
            wavelength: length * 0.25,  // Scale wavelength with length
            periods: 4.0,
            gaussian_width_factor: 8.0, // Narrower Gaussian for more linear behavior
        }
    } else {
        ChannelType::Straight
    }
});
```

## Quick Start

Add Scheme to your `Cargo.toml`:

```toml
[dependencies]
scheme = "0.1.0"
```

Basic usage:

```rust
use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure geometry parameters
    let config = GeometryConfig::default();
    
    // Create a bifurcation pattern with straight channels
    let system = create_geometry(
        (200.0, 100.0),  // box dimensions (width, height)
        &[SplitType::Bifurcation],  // split pattern
        &config,
        &ChannelTypeConfig::AllStraight,  // channel type configuration
    );
    
    // Generate visualization
    plot_geometry(&system, "schematic.png")?;
    
    Ok(())
}
```

## Examples

The library includes comprehensive examples using the modern API:

### Bifurcation Patterns
- `single_split` - Basic bifurcation
- `double_split` - Two-level bifurcation
- `triple_split` - Three-level bifurcation
- `four_split` - Four-level bifurcation
- `five_split` - Five-level bifurcation

### Trifurcation Patterns
- `single_trifurcation` - Basic trifurcation
- `double_trifurcation` - Two-level trifurcation
- `triple_trifurcation` - Three-level trifurcation
- `four_trifurcation` - Four-level trifurcation
- `five_trifurcation` - Five-level trifurcation

### Mixed Patterns
- `bifurcation_trifurcation` - Bifurcation followed by trifurcation
- `trifurcation_bifurcation` - Trifurcation followed by bifurcation
- `bifurcation_bifurcation_trifurcation` - Complex three-stage pattern
- `bifurcation_trifurcation_bifurcation` - Alternating pattern
- `trifurcation_bifurcation_trifurcation` - Alternating pattern
- `trifurcation_trifurcation_bifurcation` - Complex three-stage pattern

### Channel Type Examples
- `serpentine_demo` - Demonstrates serpentine channels in a bifurcation pattern
- `mixed_channel_types` - Shows mixed straight and serpentine channels
- `channel_type_demo` - Comprehensive demonstration of all channel type configurations
- `dynamic_serpentine_demo` - Advanced demonstration of precise endpoint alignment and custom configurations
- `gaussian_tone_burst_demo` - Demonstrates Gaussian envelope effects with different width factors

Run any example with:

```bash
cargo run --example single_split
```

## API Reference

### Core Types

- `ChannelSystem` - Represents a 2D channel layout
- `Channel` - Individual channel with type and path information
- `ChannelType` - Enum for straight or serpentine channels
- `SplitType` - Enum for bifurcation and trifurcation patterns
- `GeometryConfig` - Configuration for channel dimensions
- `ChannelTypeConfig` - Configuration for channel type selection
- `Point2D` - 2D coordinate type

### Main Functions

- `create_geometry()` - Generate channel layouts with explicit channel type configuration
- `plot_geometry()` - Export schematics as PNG images

## Architecture

Scheme focuses exclusively on 2D schematic design, having removed all 3D mesh generation, CSG operations, and CFD simulation capabilities for simplicity and performance.

## License

MIT License - see LICENSE file for details.
