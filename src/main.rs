mod data_types;
use crate::data_types::{BtAddress, DeviceData};

mod bt_magic;
use crate::bt_magic::BtMagic;

fn main() {
    let bt = BtMagic::new();
    let device_list = bt.find_devices();

    for device in device_list {
        let device_data = DeviceData {
            name: String::from_utf16_lossy(&device.szName),
            address: BtAddress::new(&device),
        };

        println!(
            "Device name: {} Connected: {} Address {}",
            device_data.name,
            bool::from(device.fConnected),
            device_data.address.str,
        );

        if let Ok(socket) = bt.connect(&device_data) {
            bt.recv(&socket);
        }
    }
}
