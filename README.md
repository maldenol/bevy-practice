# bevy-practice
Bevy (v0.18) practice. Version 0.1.0.

### Building and running
WASM release: `cargo run --profile wasm-release`
Any other release: `cargo run --release`
Development: `cargo run --features bevy/dynamic_linking,bevy/file_watcher`

### TODO
 - Make simulation speed parameter change simulation frequency.
 - Fix prediction trajectories jittering.
 - Implement scene saving and loading.
 - Unify simulation and prediction calculations.
 - Orginize the code in modules and plugins.
 - Improve UI.
