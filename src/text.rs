/// The text of the authentication prompt.
pub struct Text<'a, 'b, 'c, 'd, 'e> {
    /// The text of the authentication prompt on Android.
    pub android: AndroidText<'a, 'b, 'c>,
    /// The description of the authentication prompt on Apple devices.
    ///
    /// Appears as: "$(binary_name) is trying to $(description)".
    ///
    /// <img src="https://github.com/project-robius/robius-authentication/blob/main/assets/apple-screenshot.png">
    pub apple: &'d str,
    pub windows: &'e str,
}

/// The text of the authentication prompt on Android.
///
/// <img src="https://github.com/project-robius/robius-authentication/blob/main/assets/android-screenshot.png">
pub struct AndroidText<'a, 'b, 'c> {
    pub title: &'a str,
    pub subtitle: Option<&'b str>,
    pub description: Option<&'c str>,
}
