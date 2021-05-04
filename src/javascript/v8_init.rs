use rusty_v8 as v8;

/// This function initializes the v8 platfom.
/// Note that initialization process must be called BEFORE any API calls for v8.
pub fn init_platform() -> Result<(), ()> {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    Ok(())
}
