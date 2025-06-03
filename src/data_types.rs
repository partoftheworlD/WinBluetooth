use windows::Win32::Devices::Bluetooth::{BLUETOOTH_ADDRESS, BLUETOOTH_DEVICE_INFO};
pub struct DeviceData {
    pub name: String,
    pub address: BtAddress,
}
pub struct BtAddress {
    pub raw: BLUETOOTH_ADDRESS,
    pub str: String,
}

impl BtAddress {
    #[must_use]
    pub fn new(device: &BLUETOOTH_DEVICE_INFO) -> Self {
        Self {
            raw: device.Address,
            str: Self::bt_addr_to_string(&device.Address),
        }
    }

    /// Convert [`BLUETOOTH_ADDRESS`] (\[u8;6\]) to String like 00:1A:2B:3C:4D:5E
    #[must_use]
    pub fn bt_addr_to_string(bt_addr: &BLUETOOTH_ADDRESS) -> String {
        let mut bt_addr: [u8; 6] = unsafe { bt_addr.Anonymous.rgBytes };
        bt_addr.reverse();

        bt_addr
            .iter()
            .map(|x| format!("{x:02X}"))
            .collect::<Vec<String>>()
            .join(":")
    }
}
