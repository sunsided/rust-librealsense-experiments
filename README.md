# Experiments with librealsense2 in Rust

Toying around with / getting to know [librealsense-rust](https://github.com/jerry73204).
Don't expect anything crazy here. 🙌

## Example applications

- [enumerate/main.rs](src/enumerate/main.rs): Enumerate and list RealSense devices and perform a hardware reset on them:
  ```bash
  cargo run
  cargo run --bin enumerate
  ```

- [pointcloud/main.rs](src/pointcloud/main.rs): Capturing a point cloud from the first device and rendering it in 3D:
  ```bash
  cargo run --bin pointcloud
  ```
