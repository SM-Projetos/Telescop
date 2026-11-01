// The Servo integration requires WGPU 3.0 in the selected Slint renderer.
pub mod webview;

#[cfg(target_os = "linux")]
mod gl_bindings {
    #![allow(unsafe_op_in_unsafe_fn)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

slint::include_modules!();
