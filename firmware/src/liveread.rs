use usbd_hid::descriptor::generator_prelude::*;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
        (usage = 0x02, logical_min = 0x0) = {
            #[item_settings(data,variable,absolute)] msg_type=input;
        };
        (usage = 0x03, logical_min = 0x0) = {
            #[item_settings(data,variable,absolute)] switches=input;
        };
        (usage = 0x04, logical_min = 0x0) = {
            #[item_settings(data,variable,absolute)] buttons=input;
        };
        (usage = 0x05, logical_min = 0x0) = {
            #[item_settings(data,variable,absolute)] request=output;
        };
    }
)]

pub struct LiveReadReport {
    pub msg_type: u8,
    pub switches: u8,
    pub buttons: u8,
    pub request: u8,
}
