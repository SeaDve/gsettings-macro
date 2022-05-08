use std::path::{Path, PathBuf};

fn no_id_defined() {
    #[gsettings_macro::gen_settings(file = "./examples/test.gschema.xml")]
    // As long as your type implements `FromVariant` for `ret_type` and `ToVariant` for arg_type
    // you can define them like these.
    #[gen_settings_define(
        signature = "(ss)",
        arg_type = "(&str, &str)",
        ret_type = "(String, String)"
    )]
    #[gen_settings_define(key_name = "cache-dir", arg_type = "&Path", ret_type = "PathBuf")]
    pub struct Settings;

    let settings = Settings::new("io.github.seadve.test");

    // Note: This is just a sample object.
    // The bindings won't work as gio::ListStore don't have those properties
    let object = gio::glib::Object::new::<gio::ListStore>(&[]).expect("Failed to create object");

    settings.set_is_maximized(true);
    assert!(settings.is_maximized());
    settings.connect_is_maximized_changed(|_| {});
    settings.bind_is_maximized(&object, "prop-name").build();
    settings.create_is_maximized_action();

    settings.set_theme("dark");
    assert_eq!(settings.theme(), "dark");
    settings.connect_theme_changed(|_| {});
    settings.bind_theme(&object, "prop-name").build();
    settings.create_theme_action();

    settings.set_invalid_words(&["invalid", "words"]);
    assert_eq!(settings.invalid_words(), vec!["invalid", "words"]);
    settings.connect_invalid_words_changed(|_| {});
    settings.bind_invalid_words(&object, "prop-name").build();
    settings.create_invalid_words_action();

    settings.set_window_width(30_000);
    assert_eq!(settings.window_width(), 30_000);
    settings.connect_window_width_changed(|_| {});
    settings.bind_window_width(&object, "prop-name").build();
    settings.create_window_width_action();

    settings.set_window_height(30_000);
    assert_eq!(settings.window_height(), 30_000);
    settings.connect_window_height_changed(|_| {});
    settings.bind_window_height(&object, "prop-name").build();
    settings.create_window_height_action();

    settings.set_window_width_64(30_000);
    assert_eq!(settings.window_width_64(), 30_000);
    settings.connect_window_width_64_changed(|_| {});
    settings.bind_window_width_64(&object, "prop-name").build();
    settings.create_window_width_64_action();

    settings.set_window_height_64(30_000);
    assert_eq!(settings.window_height_64(), 30_000);
    settings.connect_window_height_64_changed(|_| {});
    settings.bind_window_height_64(&object, "prop-name").build();
    settings.create_window_height_64_action();

    settings.set_volume(1.0);
    assert_eq!(settings.volume(), 1.0);
    settings.connect_volume_changed(|_| {});
    settings.bind_volume(&object, "prop-name").build();
    settings.create_volume_action();

    settings.set_preferred_audio_source(PreferredAudioSource::DesktopAudio);
    assert_eq!(
        settings.preferred_audio_source(),
        PreferredAudioSource::DesktopAudio
    );
    settings.connect_preferred_audio_source_changed(|_| {});
    settings
        .bind_preferred_audio_source(&object, "prop-name")
        .build();
    settings.create_preferred_audio_source_action();

    settings.set_dimensions((20, 20));
    assert_eq!(settings.dimensions(), (20, 20));
    settings.connect_dimensions_changed(|_| {});
    settings.bind_dimensions(&object, "prop-name").build();
    settings.create_dimensions_action();

    // Custom type defined by the attribute
    settings.set_string_tuple(("hi", "hi"));
    assert_eq!(settings.string_tuple(), ("hi".into(), "hi".into()));
    settings.connect_string_tuple_changed(|_| {});
    settings.bind_string_tuple(&object, "prop-name").build();
    settings.create_string_tuple_action();

    // Custom type defined by the attribute
    settings.set_cache_dir(Path::new("/some_dir"));
    assert_eq!(settings.cache_dir(), PathBuf::from("/some_dir"));
    settings.connect_cache_dir_changed(|_| {});
    settings.bind_cache_dir(&object, "prop-name").build();
    settings.create_cache_dir_action();
}

fn id_defined() {
    #[gsettings_macro::gen_settings(
        file = "./examples/test.gschema.xml",
        id = "io.github.seadve.test"
    )]
    #[gen_settings_skip(signature = "(ss)")]
    pub struct Settings;

    let settings = Settings::new();

    settings.set_is_maximized(true);
    assert!(settings.is_maximized());
}

fn try_set_variant() {
    #[gsettings_macro::gen_settings(
        file = "./examples/test.gschema.xml",
        id = "io.github.seadve.test"
    )]
    #[gen_settings_skip(signature = "(ss)")]
    pub struct Settings;

    let settings = Settings::new();

    assert!(settings.try_set_is_maximized(true).is_ok());
    assert!(settings.is_maximized());
}

fn main() {
    no_id_defined();
    id_defined();
    try_set_variant();
}
