use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, COINIT_MULTITHREADED, CLSCTX_ALL, CoTaskMemFree, STGM_READ};
use windows::Win32::Media::Audio::{MMDeviceEnumerator, IMMDeviceEnumerator, eCapture, eRender, eConsole};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;

const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

/// Retrieve the friendly name of the default audio capture endpoint (microphone) on Windows.
/// Returns Ok(None) if no recording device is active or connected.
pub fn get_default_microphone_name() -> Result<Option<String>, String> {
    unsafe {
        // Initialize COM (returns S_FALSE if already initialized on this thread, which is fine)
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator = match CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ) {
            Ok(val) => val,
            Err(e) => return Err(format!("Failed to create MMDeviceEnumerator: {e}")),
        };

        let device = match enumerator.GetDefaultAudioEndpoint(eCapture, eConsole) {
            Ok(device) => device,
            Err(e) => {
                log::warn!("No default audio capture device found: {e}");
                return Ok(None);
            }
        };

        let property_store: IPropertyStore = match device.OpenPropertyStore(STGM_READ) {
            Ok(store) => store,
            Err(e) => return Err(format!("Failed to open device property store: {e}")),
        };

        let mut propvar = match property_store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME) {
            Ok(val) => val,
            Err(e) => return Err(format!("Failed to get device friendly name property: {e}")),
        };

        let psz_out = match PropVariantToStringAlloc(&propvar) {
            Ok(pwstr) => pwstr,
            Err(e) => {
                log::warn!("PropVariantToStringAlloc failed: {e}");
                let _ = windows::Win32::System::Com::StructuredStorage::PropVariantClear(&mut propvar);
                return Ok(None);
            }
        };

        // Clear the variant to avoid leaking memory
        let _ = windows::Win32::System::Com::StructuredStorage::PropVariantClear(&mut propvar);

        if !psz_out.is_null() {
            let result_string = psz_out.to_string().unwrap_or_default();
            CoTaskMemFree(Some(psz_out.0 as *const _));
            if result_string.is_empty() {
                Ok(None)
            } else {
                Ok(Some(result_string))
            }
        } else {
            Ok(None)
        }
    }
}

/// Retrieve the friendly name of the default audio render endpoint (speakers/system audio) on Windows.
/// Returns Ok(None) if no playback device is active or connected.
pub fn get_default_render_name() -> Result<Option<String>, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator = match CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ) {
            Ok(val) => val,
            Err(e) => return Err(format!("Failed to create MMDeviceEnumerator: {e}")),
        };

        let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            Ok(device) => device,
            Err(e) => {
                log::warn!("No default audio render device found: {e}");
                return Ok(None);
            }
        };

        let property_store: IPropertyStore = match device.OpenPropertyStore(STGM_READ) {
            Ok(store) => store,
            Err(e) => return Err(format!("Failed to open device property store: {e}")),
        };

        let mut propvar = match property_store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME) {
            Ok(val) => val,
            Err(e) => return Err(format!("Failed to get device friendly name property: {e}")),
        };

        let psz_out = match PropVariantToStringAlloc(&propvar) {
            Ok(pwstr) => pwstr,
            Err(e) => {
                log::warn!("PropVariantToStringAlloc failed for render device: {e}");
                let _ = windows::Win32::System::Com::StructuredStorage::PropVariantClear(&mut propvar);
                return Ok(None);
            }
        };

        let _ = windows::Win32::System::Com::StructuredStorage::PropVariantClear(&mut propvar);

        if !psz_out.is_null() {
            let result_string = psz_out.to_string().unwrap_or_default();
            CoTaskMemFree(Some(psz_out.0 as *const _));
            if result_string.is_empty() {
                Ok(None)
            } else {
                Ok(Some(result_string))
            }
        } else {
            Ok(None)
        }
    }
}

