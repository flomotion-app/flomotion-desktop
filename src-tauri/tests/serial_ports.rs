use flomotion_desktop_lib::app::serial::ports::BoardPort;
use serialport::UsbPortInfo;

#[test]
fn known_usb_chips_are_named() {
    assert_eq!(BoardPort::chip_for(0x10C4, 0xEA60), "cp210x");
    assert_eq!(BoardPort::chip_for(0x1A86, 0x7523), "ch340");
    assert_eq!(BoardPort::chip_for(0x1A86, 0x55D4), "ch9102");
    assert_eq!(BoardPort::chip_for(0x0403, 0x6001), "ftdi");
    assert_eq!(BoardPort::chip_for(0x0403, 0x6015), "ftdi");
    assert_eq!(BoardPort::chip_for(0x303A, 0x1001), "espressif");
    assert_eq!(BoardPort::chip_for(0x2341, 0x0043), "other");
}

#[test]
fn usb_info_becomes_a_board_port() {
    let usb = UsbPortInfo { vid: 0x10C4, pid: 0xEA60, serial_number: Some("0001".into()), manufacturer: None, product: Some("CP2102".into()) };
    let port = BoardPort::from_usb("COM5".into(), usb);
    assert_eq!(port.chip, "cp210x");
    assert_eq!(port.path, "COM5");
    assert_eq!(port.serial_number.as_deref(), Some("0001"));
    assert_eq!(port.product.as_deref(), Some("CP2102"));
    assert_eq!((port.vid, port.pid), (Some(0x10C4), Some(0xEA60)));
}
