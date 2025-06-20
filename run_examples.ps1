# Get the content of Cargo.toml
$cargoToml = Get-Content -Path "Cargo.toml" -Raw

# Split the file by [[example]] sections
$exampleSections = $cargoToml -split '\[\[example\]\]'

# Regex to find all example names
$regex = 'name = "([^"]+)"'

# Get the names from the matches
$exampleNames = @()
foreach ($section in $exampleSections | Select-Object -Skip 1) {
    $match = $section | Select-String -Pattern $regex
    if ($match) {
        $exampleNames += $match.Matches.Groups[1].Value
    }
}

# Loop through and run each example
foreach ($name in $exampleNames) {
    Write-Host "Running example: $name"
    $env:RUST_MIN_STACK = 8388608 # 8 MB
    cargo run --release --example $name --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error running example $name. Stopping." -ForegroundColor Red
        exit 1
    }
}

Write-Host "All examples ran successfully." 