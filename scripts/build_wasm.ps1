param(
    [string]$Crate = "game_runner",
    [string]$Target = "wasm32-unknown-unknown",
    [string]$Profile = "debug"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "Building $Crate for $Target ($Profile)…"
cargo build -p $Crate --target $Target --no-default-features --features wasm

$wasmPath = Join-Path -Path ("target/{0}/{1}" -f $Target, $Profile) -ChildPath ("{0}.wasm" -f $Crate)
if (-not (Test-Path $wasmPath)) {
    throw "Expected wasm artifact at $wasmPath"
}

$pkgDir = "web/pkg"
if (-not (Test-Path $pkgDir)) {
    New-Item -ItemType Directory -Path $pkgDir | Out-Null
}

if (Get-Command wasm-bindgen -ErrorAction SilentlyContinue) {
    Write-Host "Running wasm-bindgen → $pkgDir"
    wasm-bindgen $wasmPath --out-dir $pkgDir --target web --no-typescript
} else {
    Write-Warning "Install wasm-bindgen-cli for JS glue (cargo install wasm-bindgen-cli)."
    Copy-Item $wasmPath -Destination (Join-Path $pkgDir ("{0}.wasm" -f $Crate)) -Force
}

