use crate::define::RHICreation;
use ash::{extensions::ext::DebugUtils, vk};
use log::error;
use raw_window_handle::RawDisplayHandle;
use std::{
    borrow::Cow,
    ffi::{c_char, CStr, CString},
};

pub struct VulkanInstance {
    pub entry: ash::Entry,
    pub raw: ash::Instance,
    pub enable_debugging: bool,
    pub debug_utils: Option<DebugUtils>,
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    pub fn new(creation: RHICreation) -> anyhow::Result<Self> {
        let entry = ash::Entry::linked();

        let mut enable_debugging = creation.enable_debugging;
        if enable_debugging && !VulkanInstance::is_validation_layer_support(&entry)? {
            enable_debugging = false;
        }

        let app_name = CString::new(creation.app_name)?;
        let engine_name = CString::new("Luxseed Engine")?;
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(creation.app_version)
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_2)
            .build();

        let layer_names = VulkanInstance::get_layer_names(enable_debugging);
        let extension_names =
            VulkanInstance::get_extension_names(creation.raw_display_handle, enable_debugging)?;

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names)
            .flags(create_flags)
            .build();

        // Create Vulkan Instance
        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

        // Create Debug Utils Messenger
        let (debug_utils, debug_messenger) = if enable_debugging {
            let debug_utils = DebugUtils::new(&entry, &instance);
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(debug_utils_callback))
                .build();
            let debug_messenger =
                unsafe { debug_utils.create_debug_utils_messenger(&debug_info, None) }?;
            (Some(debug_utils), Some(debug_messenger))
        } else {
            (None, None)
        };

        Ok(Self {
            entry,
            raw: instance,
            enable_debugging,
            debug_utils,
            debug_messenger,
        })
    }

    fn is_validation_layer_support(entry: &ash::Entry) -> anyhow::Result<bool> {
        let validation_layer_name = unsafe {
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")
        };

        let mut found = false;
        for p in entry.enumerate_instance_layer_properties()? {
            let test = unsafe { CStr::from_ptr(p.layer_name.as_ptr()) };
            if test == validation_layer_name {
                found = true;
                break;
            }
        }

        if !found {
            error!("Vulkan validation layer not found: ");
            return Ok(false);
        }

        Ok(true)
    }

    fn get_layer_names(enable_validation: bool) -> Vec<*const c_char> {
        let mut layer_names = Vec::new();

        if enable_validation {
            layer_names.push(
                unsafe {
                    ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                        b"VK_LAYER_KHRONOS_validation\0",
                    )
                }
                .as_ptr(),
            );
        }

        layer_names
    }

    fn get_extension_names(
        display_handle: RawDisplayHandle,
        enable_validation_layer: bool,
    ) -> anyhow::Result<Vec<*const c_char>> {
        let mut extension_names = Vec::new();

        let window_required_extensions = ash_window::enumerate_required_extensions(display_handle)?;
        for name in window_required_extensions {
            extension_names.push(*name);
        }

        if enable_validation_layer {
            extension_names.push(DebugUtils::name().as_ptr());
        }

        Ok(extension_names)
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_messenger) = self.debug_messenger {
                DebugUtils::new(&self.entry, &self.raw)
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
            self.raw.destroy_instance(None);
        }
    }
}

unsafe extern "system" fn debug_utils_callback(
    _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let d = *p_callback_data;

    let message_id_number = d.message_id_number;
    let message_id_name = if d.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(d.p_message_id_name).to_string_lossy()
    };
    let message = if d.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(d.p_message).to_string_lossy()
    };

    println!("MessageID: {message_id_name} {message_id_number}\nMessage: {message}\n\n");

    vk::FALSE
}
