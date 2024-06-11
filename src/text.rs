/// The text contents displayed by an authentication prompt.
pub struct Text<'a, 'b, 'c, 'd, 'e, 'f> {
    /// The text of the authentication prompt on Android.
    pub android: AndroidText<'a, 'b, 'c>,
    /// The description of the authentication prompt on Apple devices.
    ///
    /// Appears as "$(binary_name) is trying to $(description)".
    pub apple: &'d str,
    /// The description of the authentication prompt on Windows.
    pub windows: WindowsText<'e, 'f>,
}

/// The text of the authentication prompt on Android.
pub struct AndroidText<'a, 'b, 'c> {
    pub title: &'a str,
    pub subtitle: Option<&'b str>,
    pub description: Option<&'c str>,
}

/// The text of the authentication prompt on Windows,
/// including a title ("caption") and description ("message").
pub struct WindowsText<'a, 'b> {
    #[allow(dead_code)]
    pub(crate) title: &'a str,
    #[allow(dead_code)]
    pub(crate) description: &'b str,
}

impl<'a, 'b> WindowsText<'a, 'b> {
    /// Creates a new `WindowsText` instance.
    ///
    /// The `title` ("caption") will be truncated to 128 bytes,
    /// and the `description` ("message") will be truncated to 1024 bytes.
    pub const fn new(title: &'a str, description: &'b str) -> Self {
        Self { title, description }
    }
}
