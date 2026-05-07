const VID: u16 = 0x1209;
const PID: u16 = 0xd9d0;

fn main() {
    let api = hidapi::HidApi::new().unwrap();

    let mut devices = api
        .device_list()
        .filter(|val| val.product_id() == PID && val.vendor_id() == VID);

    let device = devices.next().unwrap().open_device(&api).unwrap();

    device.write(&[0x00, 0x02]).unwrap();
}
