# run_3d_examples.ps1
# This script runs all examples related to 3D geometry and operations.

# Set a larger stack size to prevent overflows in complex CSG operations.
$env:RUST_MIN_STACK = 8388608 # 8 MB

$all_3d_examples = @(
    "basic_box",
    "basic_cone",
    "basic_cylinder",
    "basic_sphere",
    "basic_torus",
    "schematic_to_3d",
    "intersection_test",
    "xor_test",
    "union_test",
    "subtract_test"
)

$examples_to_run = if ($args.Count -gt 0) {
    $args
} else {
    $all_3d_examples
}

foreach ($name in $examples_to_run) {
    Write-Host "Running 3D example: $name"
    cargo run --release --example $name --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error running example $name. Stopping." -ForegroundColor Red
        exit 1
    }
}

$plural = if ($examples_to_run.Count -gt 1) { "s" } else { "" }
Write-Host "All specified 3D example$plural ran successfully." -ForegroundColor Green 