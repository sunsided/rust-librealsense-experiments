# Experiments with librealsense2 in Rust

Toying around with / getting to know [librealsense-rust](https://github.com/jerry73204).
Don't expect anything crazy here. 🙌

## Example applications

- [enumerate.rs](src/enumerate.rs): Enumerate and list RealSense devices and perform a hardware reset on them:
  ```bash
  cargo run
  cargo run --bin enumerate
  ```

- [pointcloud.rs](src/pointcloud.rs): Capturing a point cloud from the first device and rendering it in 3D:
  ```bash
  cargo run --bin pointcloud
  ```
