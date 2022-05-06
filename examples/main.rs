fn main() {
    use gio::prelude::*;

    #[gsettings_macro::gen_settings(
        file = "./examples/test.gschema.xml",
        id = "io.github.sadve.test"
    )]
    pub struct Settings;

    let settings = Settings::new();

    settings
        .set_is_maximized(true)
        .expect("key is not writable");
    assert!(settings.is_maximized());

    settings
        .set_keycode("very secure password")
        .expect("key is not writable");
    assert_eq!(settings.keycode(), "very secure password");

    settings
        .set_invalid_words(&["invalid", "words"])
        .expect("key is not writable");
    assert_eq!(settings.invalid_words(), vec!["invalid", "words"]);

    settings
        .set_window_width(30_000)
        .expect("key is not writable");
    settings.window_width();
    assert_eq!(settings.window_width(), 30_000);
}
