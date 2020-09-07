use anyhow::ensure;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Looking for RealSense devices");
    let ctx = realsense_rust::Context::new()?;
    let devices = ctx.query_devices(None)?;
    let mut devices_found: bool = false;

    for device in devices {
        let device = device.unwrap();
        let name = device.name().unwrap().unwrap();
        let sn = device.serial_number().unwrap().unwrap();
        println!("Found {} SN {}", name, sn);
        devices_found = true;

        // It's from the demo code, but why not.
        device.hardware_reset()?;
    }
    ensure!(devices_found, "No devices found");
    Ok(())
}
