use std::rc::Rc;

use windows::{
    Win32::{
        Devices::Bluetooth::{
            AF_BTH, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS, BTHPROTO_RFCOMM,
            BluetoothFindFirstDevice, BluetoothFindNextDevice, HBLUETOOTH_DEVICE_FIND,
            RFCOMM_PROTOCOL_UUID16, SOCKADDR_BTH,
        },
        Foundation::HANDLE,
        Networking::WinSock::{
            FIONBIO, SEND_RECV_FLAGS, SOCK_STREAM, SOCKADDR, SOCKET, WSA_ERROR, WSACleanup,
            WSADATA, WSAGetLastError, WSAStartup, connect, ioctlsocket, recv, socket,
        },
    },
    core::GUID,
};

use crate::data_types::DeviceData;

#[derive(Default)]
pub struct BtMagic;

impl BtMagic {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn find_devices(&self) -> Vec<BLUETOOTH_DEVICE_INFO> {
        let bt_device_find = HBLUETOOTH_DEVICE_FIND::default();

        let mut device_list = Vec::<BLUETOOTH_DEVICE_INFO>::new();

        let bt_device_search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: true.into(),
            fReturnRemembered: false.into(),
            fReturnUnknown: true.into(),
            fReturnConnected: true.into(),
            fIssueInquiry: true.into(),
            cTimeoutMultiplier: 2,
            hRadio: HANDLE::default(),
        };

        let mut device_info = BLUETOOTH_DEVICE_INFO {
            dwSize: u32::try_from(size_of::<BLUETOOTH_DEVICE_INFO>()).unwrap(),
            ..Default::default()
        };

        unsafe {
            match BluetoothFindFirstDevice(&bt_device_search_params, &mut device_info) {
                Ok(_) => device_list.push(device_info),
                Err(_) => panic!("Device not found!"),
            }

            // BluetoothFindNextDevice should work, but I don't have any other devices to test this.
            while let Ok(()) = BluetoothFindNextDevice(bt_device_find, &mut device_info) {
                device_list.push(device_info);
            }
            device_list
        }
    }

    #[must_use]
    fn wsa_startup(&self) -> bool {
        let version = self.makeword(2, 2);
        let mut wsa_data = WSADATA::default();

        unsafe {
            let wsa_startup_error = WSAStartup(version, &mut wsa_data);
            if wsa_startup_error != 0 {
                println!("WSAStartup failed with error: {:?}", WSAGetLastError());
                return false;
            }
        }
        true
    }

    pub fn connect(&self, device_data: &DeviceData) -> Result<SOCKET, WSA_ERROR> {
        // Without first running WSAStartup Bt in windows does not work
        if self.wsa_startup() {
            let socket = unsafe {
                socket(
                    AF_BTH.into(),
                    SOCK_STREAM,
                    BTHPROTO_RFCOMM.try_into().unwrap(),
                )
            };

            if socket.is_err() {
                unsafe {
                    WSACleanup();
                }
                return Err(unsafe { WSAGetLastError() });
            }

            let rc_socket = match socket {
                Ok(s) => Rc::new(s),
                Err(_) => return Err(unsafe { WSAGetLastError() }),
            };

            let bt_socket_address = SOCKADDR_BTH {
                addressFamily: AF_BTH,
                serviceClassId: self.uuid16_to_guid(RFCOMM_PROTOCOL_UUID16),
                port: 0,
                btAddr: unsafe { device_data.address.raw.Anonymous.ullLong },
            };

            // (SOCKADDR*)bt_socket_address
            let sockaddr_ptr: *const SOCKADDR = (&raw const bt_socket_address).cast::<SOCKADDR>();

            unsafe {
                if connect(*rc_socket, sockaddr_ptr, size_of::<SOCKADDR_BTH>() as i32) != 0 {
                    let err = WSAGetLastError();
                    println!("{err:?} Could not connect!");
                    return Err(err);
                }

                let mut non_blocking_mode = 1u32;
                if ioctlsocket(*rc_socket, FIONBIO, &mut non_blocking_mode) != 0 {
                    let err = WSAGetLastError();
                    println!("{err:?} Could not set socket to be non-blocking");
                    return Err(err);
                }
            }
            Ok(*rc_socket)
        } else {
            Err(unsafe { WSAGetLastError() })
        }
    }

    pub fn recv(&self, socket: &SOCKET) {
        loop {
            let mut recv_buffer = [0u8; 64];
            let recv_result = unsafe { recv(*socket, &mut recv_buffer, SEND_RECV_FLAGS(0)) };

            if recv_result < 0 {
                continue;
            }
            if recv_result > 0 {
                println!("Bytes received: {recv_result:?} bytes\nMessage - {recv_buffer:?}");
            }
            if recv_result == 0 {
                println!("Connection closed");
                break;
            }
        }
    }

    #[inline]
    fn makeword(&self, lo: u16, hi: u16) -> u16 {
        (lo & 0xff) | ((hi & 0xff) << 8)
    }

    /// You need to define `RFCOMM_PROTOCOL_UUID` manually.
    /// Because stupid [`winapi::shared::bthdef::RFCOMM_PROTOCOL_UUID`] 
    /// is not the same [`GUID`] type as [`GUID`] in windows crate.
    /// Or do a stupid type conversion with unsafe dereferencing
    ///
    /// # Example:
    /// ```
    /// use winapi::shared::bthdef::RFCOMM_PROTOCOL_UUUID;
    /// let good_guid = &RFCOMM_PROTOCOL_UUID as *const _ as *const windows::core::GUID;
    ///
    /// ```
    #[inline]
    fn uuid16_to_guid(&self, uuid16: u32) -> GUID {
        GUID {
            data1: uuid16,
            data2: 0x0000,
            data3: 0x1000,
            data4: [0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB],
        }
    }
}
