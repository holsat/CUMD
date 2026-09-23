use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use objc::{msg_send, sel, sel_impl};
use core_graphics::display::*;
use crate::hal::driver::{DisplayInfo, DriverError};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGGetOnlineDisplayList(
        max_displays: u32,
        online_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> CGError;
}

static VIRTUAL_DISPLAYS: OnceLock<Mutex<HashMap<u32, usize>>> = OnceLock::new();

fn get_registry() -> &'static Mutex<HashMap<u32, usize>> {
    VIRTUAL_DISPLAYS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_virtual_display(
    width: u32,
    height: u32,
    name: Option<&str>,
) -> Result<DisplayInfo, DriverError> {
    let display_name = name.unwrap_or("CUMD-Virtual-Display");

    let cls_vd = objc::runtime::Class::get("CGVirtualDisplay")
        .ok_or_else(|| DriverError::OsError("CGVirtualDisplay class not found".to_string()))?;
    let cls_desc = objc::runtime::Class::get("CGVirtualDisplayDescriptor")
        .ok_or_else(|| DriverError::OsError("CGVirtualDisplayDescriptor class not found".to_string()))?;
    let cls_settings = objc::runtime::Class::get("CGVirtualDisplaySettings")
        .ok_or_else(|| DriverError::OsError("CGVirtualDisplaySettings class not found".to_string()))?;
    let cls_mode = objc::runtime::Class::get("CGVirtualDisplayMode")
        .ok_or_else(|| DriverError::OsError("CGVirtualDisplayMode class not found".to_string()))?;
    let cls_str = objc::runtime::Class::get("NSString")
        .ok_or_else(|| DriverError::OsError("NSString class not found".to_string()))?;
    let cls_arr = objc::runtime::Class::get("NSArray")
        .ok_or_else(|| DriverError::OsError("NSArray class not found".to_string()))?;

    unsafe {
        // 1. Allocate descriptor
        let desc: cocoa::base::id = msg_send![cls_desc, alloc];
        let desc: cocoa::base::id = msg_send![desc, init];
        if desc.is_null() {
            return Err(DriverError::OsError("Failed to init CGVirtualDisplayDescriptor".to_string()));
        }

        // Set name
        let c_name = std::ffi::CString::new(display_name).unwrap_or_default();
        let ns_name: cocoa::base::id = msg_send![cls_str, stringWithUTF8String: c_name.as_ptr()];
        let _: () = msg_send![desc, setName: ns_name];
        let _: () = msg_send![desc, setMaxPixelsWide: width];
        let _: () = msg_send![desc, setMaxPixelsHigh: height];
        let _: () = msg_send![desc, setProductID: 0x5644u32]; // 'VD'
        let _: () = msg_send![desc, setVendorID: 0x4355u32];  // 'CU'

        // 2. Initialize CGVirtualDisplay
        let vd: cocoa::base::id = msg_send![cls_vd, alloc];
        let vd: cocoa::base::id = msg_send![vd, initWithDescriptor: desc];
        if vd.is_null() {
            return Err(DriverError::OsError("Failed to init CGVirtualDisplay with descriptor".to_string()));
        }

        // 3. Configure mode (width x height @ 60.0 Hz)
        let mode: cocoa::base::id = msg_send![cls_mode, alloc];
        let mode: cocoa::base::id = msg_send![mode, initWithWidth: width as usize height: height as usize refreshRate: 60.0f64];

        // 4. Configure settings
        let settings: cocoa::base::id = msg_send![cls_settings, alloc];
        let settings: cocoa::base::id = msg_send![settings, init];
        let modes_arr: cocoa::base::id = msg_send![cls_arr, arrayWithObject: mode];
        let _: () = msg_send![settings, setModes: modes_arr];

        // 5. Apply settings to activate virtual display
        let applied: bool = msg_send![vd, applySettings: settings];
        if !applied {
            let _: () = msg_send![vd, release];
            return Err(DriverError::OsError("Failed to apply settings to CGVirtualDisplay".to_string()));
        }

        let display_id: u32 = msg_send![vd, displayID];

        let mut registry = get_registry().lock().unwrap();
        registry.insert(display_id, vd as usize);

        // Small delay for Quartz Window Server to register new online display
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(DisplayInfo {
            display_id,
            name: display_name.to_string(),
            width,
            height,
            is_virtual: true,
            is_main: false,
            refresh_rate: 60.0,
        })
    }
}

pub fn destroy_virtual_display(display_id: u32) -> Result<(), DriverError> {
    let mut registry = get_registry().lock().unwrap();
    if let Some(vd_ptr) = registry.remove(&display_id) {
        unsafe {
            let vd = vd_ptr as cocoa::base::id;
            let _: () = msg_send![vd, release];
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        Ok(())
    } else {
        Err(DriverError::OsError(format!("Virtual display with ID {} not found", display_id)))
    }
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, DriverError> {
    let registry = get_registry().lock().unwrap();
    let mut online_displays = [0u32; 32];
    let mut display_count = 0u32;

    unsafe {
        let err = CGGetOnlineDisplayList(
            32,
            online_displays.as_mut_ptr(),
            &mut display_count,
        );

        if err != 0 {
            return Err(DriverError::OsError(format!("CGGetOnlineDisplayList failed with code {}", err)));
        }

        let mut displays = Vec::new();
        for i in 0..(display_count as usize) {
            let did = online_displays[i];
            let width = CGDisplayPixelsWide(did) as u32;
            let height = CGDisplayPixelsHigh(did) as u32;
            let is_main = CGDisplayIsMain(did) != 0;
            let is_virtual = registry.contains_key(&did);
            let name = if is_virtual {
                format!("Virtual Display ({})", did)
            } else if is_main {
                "Main Display".to_string()
            } else {
                format!("Display ({})", did)
            };

            displays.push(DisplayInfo {
                display_id: did,
                name,
                width,
                height,
                is_virtual,
                is_main,
                refresh_rate: 60.0,
            });
        }

        Ok(displays)
    }
}
