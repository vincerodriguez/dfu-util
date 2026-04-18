use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use crate::error::DfuError; 
use crate::protocol::{USB_CLASS_APP_SPECIFIC, USB_SUBCLASS_DFU};

#[derive(Debug)]
pub struct DfuDevice {
    pub vendor_id: u16, 
    pub product_id: u16,
    pub bus: u8,
    pub address: u8,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial: Option<String>,
}

pub fn find_dfu_devices() -> Result<Vec<DfuDevice>, DfuError> {
    let context = Context::new()?;
    let devices = context.devices()?;

    let mut dfu_devices = Vec::new();

    for device in devices.iter() {
        let descriptor = device.device_descriptor()?;

        if is_dfu_device(&device, &descriptor)? {
            let dfu = read_device_info(&device, &descriptor)?;
            dfu_devices.push(dfu);
        }
    }

    Ok(dfu_devices)
}

fn is_dfu_device(
    device: &Device<Context>, 
    descriptor: &DeviceDescriptor, 
) -> Result<bool, DfuError> {
    for config_index in 0..descriptor.num_configurations(){
        let config = device.config_descriptor(config_index)?;

        for interface in config.interfaces() {
            for setting in interface.descriptors() {
                if setting.class_code() == USB_CLASS_APP_SPECIFIC
                && setting.sub_class_code() == USB_SUBCLASS_DFU
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn read_device_info(
    device: &Device<Context>,
    descriptor: &DeviceDescriptor,
) -> Result<DfuDevice, DfuError> {


    let handle = device.open()?;

    let manufacturer = handle
    .read_manufacturer_string_ascii(descriptor)
    .ok();

    let product = handle
    .read_product_string_ascii(descriptor)
    .ok();

    let serial = handle
    .read_serial_number_string_ascii(descriptor)
    .ok();

    Ok(DfuDevice {
        vendor_id: descriptor.vendor_id(),
        product_id: descriptor.product_id(),
        bus: device.bus_number(),
        address: device.address(),
        manufacturer, 
        product, 
        serial,
    })
}