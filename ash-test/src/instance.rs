use ash::version::EntryV1_0;
use ash::vk;

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let message_type = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, message_type, message);

    vk::FALSE
}

fn get_debug_utils_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: std::ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            //| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(debug_callback),
        p_user_data: std::ptr::null_mut(),
    }
}

pub fn create_instance(entry: &ash::Entry) -> ash::Instance {
    let app_name = std::ffi::CString::new("Vulkan app").unwrap();
    let engine_name = std::ffi::CString::new("Vulkan Engine").unwrap();

    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: std::ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: crate::constants::APPLICATION_VERSION,
        p_engine_name: engine_name.as_ptr(),
        engine_version: crate::constants::ENGINE_VERSION,
        api_version: crate::constants::API_VERSION,
    };

    let debug_utils_create_info = get_debug_utils_messenger_create_info();

    let layers = vec!["VK_LAYER_KHRONOS_validation"];
    let layers_raw: Vec<std::ffi::CString> = layers
        .iter()
        .map(|layer_name| std::ffi::CString::new(*layer_name).unwrap())
        .collect();
    let layers_ptr: Vec<*const i8> = layers_raw.iter().map(|layer_name| layer_name.as_ptr()).collect();

    let extension_names = vec![
        ash::extensions::khr::Surface::name().as_ptr(),
        ash::extensions::khr::Win32Surface::name().as_ptr(),
        ash::extensions::ext::DebugUtils::name().as_ptr(),
    ];

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const std::ffi::c_void,
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: layers_ptr.as_ptr(),
        enabled_layer_count: layers_ptr.len() as u32,
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,
    };

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance!")
    };

    instance
}

pub fn create_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_utils = ash::extensions::ext::DebugUtils::new(entry, instance);
    let create_info = get_debug_utils_messenger_create_info();
    let utils_messenger = unsafe {
        debug_utils
            .create_debug_utils_messenger(&create_info, None)
            .expect("Debug Utils Callback")
    };
    (debug_utils, utils_messenger)
}
