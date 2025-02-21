# Raytracing-Rs

Raytracing-Rs is a GPU-accelerated, multi-threaded 2D ray tracing application built in Rust using the [pixels](https://crates.io/crates/pixels) crate. It demonstrates basic light and shadow rendering with real-time performance optimizations.

## Features
- **Real-Time Ray Tracing:** Efficient light and shadow calculations using parallel processing with Rayon.
- **GPU Acceleration:** Utilizes WGPU for high-performance rendering.
- **System Monitoring:** Displays real-time FPS, CPU usage, and memory statistics.
- **Interactive Lighting:** Drag the light source to see dynamic shadow effects.
- **Cross-Platform Compatibility:** Runs on Windows, macOS, and Linux.

## Screenshots
*Coming soon!*

## Installation

1. Ensure you have Rust installed. If not, get it from [rust-lang.org](https://www.rust-lang.org/tools/install).
2. Clone the repository:
    ```bash
    git clone https://github.com/your-username/vfxwq-raytracing-rs.git
    cd vfxwq-raytracing-rs
    ```
3. Build and run the application:
    ```bash
    cargo run --release
    ```

## Dependencies
- [wgpu](https://crates.io/crates/wgpu) - Low-level graphics API for GPU acceleration.
- [pixels](https://crates.io/crates/pixels) - Minimal pixel buffer for Rust.
- [rayon](https://crates.io/crates/rayon) - Data parallelism library for multi-threaded processing.
- [tokio](https://crates.io/crates/tokio) - Asynchronous runtime for Rust.
- [winit](https://crates.io/crates/winit) - Cross-platform window creation and event handling.
- [sysinfo](https://crates.io/crates/sysinfo) - System monitoring (CPU, memory stats).

## How It Works
The application renders a bouncing circle that casts shadows when illuminated by a draggable light source. It uses:
- **Ray tracing** to calculate light and shadow positions.
- **Parallel computation** with Rayon to enhance rendering performance.
- **System monitoring** to provide real-time feedback on FPS, CPU, and memory usage.

## Performance Optimizations
This project is optimized for high performance:
- **Link-Time Optimization:** Using `lto = "fat"` to reduce function duplication.
- **Codegen Units:** Set to `1` to allow for whole-program optimization.
- **Panic Abort:** To reduce binary size and overhead.
- **Release Configurations:** `opt-level = 3` for maximum performance.

## Contribution
Contributions are welcome! Feel free to open issues or submit pull requests to enhance the project.

## License
This project is licensed under the MIT License.

## Acknowledgements
Special thanks to the developers of the Rust ecosystem and the [pixels](https://crates.io/crates/pixels) crate for making real-time rendering in Rust easier.

## Closure
Thanks for your time! Feel free to explore this project and use it as you need.