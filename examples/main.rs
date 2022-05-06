fn no_id_defined() {
    #[gsettings_macro::gen_settings(file = "./examples/test.gschema.xml")]
    pub struct Settings;

    let settings = Settings::new("io.github.seadve.test");

    settings.set_is_maximized(true);
    assert!(settings.is_maximized());

    settings.set_theme("dark");
    assert_eq!(settings.theme(), "dark");

    settings.set_invalid_words(&["invalid", "words"]);
    assert_eq!(settings.invalid_words(), vec!["invalid", "words"]);

    settings.set_window_width(30_000);
    assert_eq!(settings.window_width(), 30_000);

    settings.set_preferred_audio_source(PreferredAudioSource::DesktopAudio);
    assert_eq!(
        settings.preferred_audio_source(),
        PreferredAudioSource::DesktopAudio
    );
}

fn id_defined() {
    #[gsettings_macro::gen_settings(
        file = "./examples/test.gschema.xml",
        id = "io.github.seadve.test"
    )]
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
