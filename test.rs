// Generated with gsettings-codegen v0.1.0

#[derive(Debug, Clone)]
pub struct Settings(gio::Settings);
impl Settings {
    pub fn new() -> Self {
        Self(gio::Settings::new("io.github.seadve.test"))
    }
    pub fn create_action(&self, key: &str) -> gio::Action {
        self.0.create_action(key)
    }
    pub fn is_maximized(&self) -> bool {
        self.0.boolean("is-maximized")
    }
    pub fn set_is_maximized(&self, value: bool) -> Result<(), glib::BoolError> {
        self.0.set_boolean("is-maximized", value)
    }
    pub fn history(&self) -> glib::GString {
        self.0.string("history")
    }
    pub fn set_history(&self, value: &str) -> Result<(), glib::BoolError> {
        self.0.set_string("history", value)
    }
    pub fn invalid_words(&self) -> Vec<glib::GString> {
        self.0.strv("invalid-words")
    }
    pub fn set_invalid_words(&self, value: &[&str]) -> Result<(), glib::BoolError> {
        self.0.set_strv("invalid-words", value)
    }
    pub fn window_width(&self) -> i32 {
        self.0.int("window-width")
    }
    pub fn set_window_width(&self, value: i32) -> Result<(), glib::BoolError> {
        self.0.set_int("window-width", value)
    }
    pub fn window_height(&self) -> u32 {
        self.0.uint("window-height")
    }
    pub fn set_window_height(&self, value: u32) -> Result<(), glib::BoolError> {
        self.0.set_uint("window-height", value)
    }
    pub fn window_width_64(&self) -> i64 {
        self.0.int64("window-width-64")
    }
    pub fn set_window_width_64(&self, value: i64) -> Result<(), glib::BoolError> {
        self.0.set_int64("window-width-64", value)
    }
    pub fn window_height_64(&self) -> u64 {
        self.0.uint64("window-height-64")
    }
    pub fn set_window_height_64(&self, value: u64) -> Result<(), glib::BoolError> {
        self.0.set_uint64("window-height-64", value)
    }
    pub fn volume(&self) -> f64 {
        self.0.double("volume")
    }
    pub fn set_volume(&self, value: f64) -> Result<(), glib::BoolError> {
        self.0.set_double("volume", value)
    }
    pub fn preferred_audio_source(&self) -> glib::GString {
        self.0.string("preferred-audio-source")
    }
    pub fn set_preferred_audio_source(&self, value: &str) -> Result<(), glib::BoolError> {
        self.0.set_string("preferred-audio-source", value)
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
