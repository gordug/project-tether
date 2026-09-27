fn main() {
    let mut config = slint_build::CompilerConfiguration::new();
    config = config.with_debug_info(true);
    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
