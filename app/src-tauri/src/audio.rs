use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputKind {
    Speakers,
    Headphones,
    Headset,
    Display,
    Controller,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOutput {
    pub id: String,
    pub name: String,
    pub detail: Option<String>,
    pub kind: OutputKind,
    pub current: bool,
}

const CONTROLLER_NAMES: [&str; 3] = ["dualsense", "wireless controller", "dualshock"];

pub fn kind_of(form_factor: Option<u32>, detail: Option<&str>) -> OutputKind {
    let lower = detail.unwrap_or_default().to_lowercase();
    if CONTROLLER_NAMES.iter().any(|name| lower.contains(name)) {
        return OutputKind::Controller;
    }
    match form_factor {
        Some(1) => OutputKind::Speakers,
        Some(3) => OutputKind::Headphones,
        Some(5 | 6) => OutputKind::Headset,
        Some(9) => OutputKind::Display,
        _ => OutputKind::Other,
    }
}

#[cfg(windows)]
pub use platform::{list, set_current};

#[cfg(not(windows))]
pub fn list() -> Result<Vec<AudioOutput>, String> {
    Ok(Vec::new())
}

#[cfg(not(windows))]
pub fn set_current(_id: &str) -> Result<(), String> {
    Err("Switching the sound output only works on Windows.".into())
}

#[cfg(windows)]
#[allow(non_snake_case)]
mod platform {
    use windows::Win32::Devices::FunctionDiscovery::{PKEY_Device_DeviceDesc, PKEY_Device_FriendlyName, PKEY_DeviceInterface_FriendlyName};
    use windows::Win32::Foundation::PROPERTYKEY;
    use windows::Win32::Media::Audio::{
        DEVICE_STATE_ACTIVE, ERole, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, PKEY_AudioEndpoint_FormFactor, eCommunications,
        eConsole, eMultimedia, eRender,
    };
    use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
    use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, STGM_READ};
    use windows::Win32::System::Variant::VT_UI4;
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use windows::core::{GUID, HRESULT, IUnknown, IUnknown_Vtbl, PCWSTR, PWSTR};

    use super::{AudioOutput, kind_of};

    const POLICY_CONFIG_CLIENT: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

    #[windows::core::interface("f8679f50-850a-41cf-9c72-430f290290c8")]
    unsafe trait IPolicyConfig: IUnknown {
        fn GetMixFormat(&self) -> HRESULT;
        fn GetDeviceFormat(&self) -> HRESULT;
        fn ResetDeviceFormat(&self) -> HRESULT;
        fn SetDeviceFormat(&self) -> HRESULT;
        fn GetProcessingPeriod(&self) -> HRESULT;
        fn SetProcessingPeriod(&self) -> HRESULT;
        fn GetShareMode(&self) -> HRESULT;
        fn SetShareMode(&self) -> HRESULT;
        fn GetPropertyValue(&self) -> HRESULT;
        fn SetPropertyValue(&self) -> HRESULT;
        fn SetDefaultEndpoint(&self, device_id: PCWSTR, role: ERole) -> HRESULT;
        fn SetEndpointVisibility(&self) -> HRESULT;
    }

    struct Com {
        owned: bool,
    }

    impl Com {
        fn start() -> Self {
            let owned = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
            Self { owned }
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            if self.owned {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn message(error: windows::core::Error) -> String {
        format!("Windows couldn't read the sound outputs ({}).", error.message())
    }

    fn take_string(raw: PWSTR) -> Option<String> {
        if raw.is_null() {
            return None;
        }
        let text = unsafe { raw.to_string() }.ok();
        unsafe { CoTaskMemFree(Some(raw.0 as *const _)) };
        text.filter(|value| !value.trim().is_empty())
    }

    fn text_property(store: &IPropertyStore, key: &PROPERTYKEY) -> Option<String> {
        let mut value = unsafe { store.GetValue(key) }.ok()?;
        let text = unsafe { PropVariantToStringAlloc(&value) }.ok().and_then(take_string);
        let _ = unsafe { PropVariantClear(&mut value) };
        text
    }

    fn number_property(store: &IPropertyStore, key: &PROPERTYKEY) -> Option<u32> {
        let mut value = unsafe { store.GetValue(key) }.ok()?;
        let number = unsafe {
            let inner = &value.Anonymous.Anonymous;
            (inner.vt == VT_UI4).then(|| inner.Anonymous.ulVal)
        };
        let _ = unsafe { PropVariantClear(&mut value) };
        number
    }

    fn device_id(device: &IMMDevice) -> Option<String> {
        take_string(unsafe { device.GetId() }.ok()?)
    }

    fn enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
    }

    pub fn list() -> Result<Vec<AudioOutput>, String> {
        let _com = Com::start();
        let devices = enumerator().map_err(message)?;
        let current = unsafe { devices.GetDefaultAudioEndpoint(eRender, eConsole) }.ok().and_then(|device| device_id(&device));
        let collection = unsafe { devices.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) }.map_err(message)?;
        let count = unsafe { collection.GetCount() }.map_err(message)?;
        let mut outputs = Vec::with_capacity(count as usize);
        for index in 0..count {
            let Ok(device) = (unsafe { collection.Item(index) }) else {
                continue;
            };
            let Some(id) = device_id(&device) else {
                continue;
            };
            let Ok(store) = (unsafe { device.OpenPropertyStore(STGM_READ) }) else {
                continue;
            };
            let detail = text_property(&store, &PKEY_DeviceInterface_FriendlyName);
            let name = text_property(&store, &PKEY_Device_DeviceDesc)
                .or_else(|| text_property(&store, &PKEY_Device_FriendlyName))
                .unwrap_or_else(|| "Sound output".into());
            let kind = kind_of(number_property(&store, &PKEY_AudioEndpoint_FormFactor), detail.as_deref());
            outputs.push(AudioOutput { current: current.as_deref() == Some(id.as_str()), id, name, detail, kind });
        }
        Ok(outputs)
    }

    pub fn set_current(id: &str) -> Result<(), String> {
        let _com = Com::start();
        let policy: IPolicyConfig = unsafe { CoCreateInstance(&POLICY_CONFIG_CLIENT, None, CLSCTX_ALL) }
            .map_err(|error| format!("Windows couldn't switch the sound output ({}).", error.message()))?;
        let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        for role in [eConsole, eMultimedia, eCommunications] {
            unsafe { policy.SetDefaultEndpoint(PCWSTR(wide.as_ptr()), role) }
                .ok()
                .map_err(|error| format!("Windows couldn't switch the sound output ({}).", error.message()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_kind_of_output() {
        assert_eq!(kind_of(Some(9), Some("LG TV SSCR2")), OutputKind::Display);
        assert_eq!(kind_of(Some(3), Some("USB Audio 2.0")), OutputKind::Headphones);
        assert_eq!(kind_of(Some(1), Some("DualSense Wireless Controller")), OutputKind::Controller);
        assert_eq!(kind_of(Some(5), None), OutputKind::Headset);
        assert_eq!(kind_of(None, None), OutputKind::Other);
    }

    #[test]
    #[ignore = "reads the sound outputs of this PC"]
    fn lists_the_sound_outputs_of_this_pc() {
        for output in list().unwrap() {
            println!("{:?} {} / {:?} current={}", output.kind, output.name, output.detail, output.current);
        }
    }
}
