fn main() {
    print!(
        "{}",
        oxfunc_core::registry::render_registry_metadata_csv(
            oxfunc_core::registry::builtin_registry()
        )
    );
}
