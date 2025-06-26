# Scheme - 2D Microfluidic Schematic Design Library

A focused Rust library for designing 2D microfluidic schematics with support for bifurcation and trifurcation patterns, channel layout algorithms, and schematic visualization.

## Features

- **Bifurcation Patterns**: Generate single to five-level bifurcation channel layouts
- **Trifurcation Patterns**: Create single to five-level trifurcation designs  
- **Mixed Patterns**: Combine bifurcation and trifurcation in complex layouts
- **2D Visualization**: Export schematics as PNG images using plotters
- **Configurable Geometry**: Customize channel dimensions and wall clearances

## Quick Start

Add Scheme to your `Cargo.toml`:

```toml
[dependencies]
scheme = "0.1.0"
```

Basic usage:

```rust
use scheme::{
    geometry::{create_geometry, SplitType},
    config::GeometryConfig,
    visualizations::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure geometry parameters
    let config = GeometryConfig::default();
    
    // Create a bifurcation pattern
    let system = create_geometry(
        (200.0, 100.0),  // box dimensions (width, height)
        &[SplitType::Bifurcation],  // split pattern
        &config,
    );
    
    // Generate visualization
    plot_geometry(&system, "schematic.png")?;
    
    Ok(())
}
```

## Examples

The library includes 16 comprehensive examples:

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

Run any example with:

```bash
cargo run --example single_split
```

## API Reference

### Core Types

- `ChannelSystem` - Represents a 2D channel layout
- `SplitType` - Enum for bifurcation and trifurcation patterns
- `GeometryConfig` - Configuration for channel dimensions
- `Point2D` - 2D coordinate type

### Main Functions

- `create_geometry()` - Generate channel layouts from split patterns
- `plot_geometry()` - Export schematics as PNG images

## Architecture

Scheme focuses exclusively on 2D schematic design, having removed all 3D mesh generation, CSG operations, and CFD simulation capabilities for simplicity and performance.

## License

MIT License - see LICENSE file for details.
