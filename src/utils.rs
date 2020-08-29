use image;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// read the texture into memory as a whole bloc (i.e. no streaming)
fn read_image(buffer: &[u8]) -> Option<image::RgbImage> {
    image::load_from_memory(buffer)
        .map(|img| img.flipv().to_rgb())
        .ok()
}
