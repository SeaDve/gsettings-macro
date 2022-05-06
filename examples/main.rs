fn main() {
    use gio::prelude::*;

    #[gsettings_macro::gen_settings(
        file = "./examples/test.gschema.xml",
        id = "io.github.sadve.test"
    )]
    pub struct Settings;

    let settings = Settings::new();

    settings.set_window_width(3).unwrap();
    settings.window_width();

    settings.set_window_height(3).unwrap();
    settings.window_width();

    settings.set_invalid_words(&["test"]).unwrap();
    settings.invalid_words();
}
