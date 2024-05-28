pub struct Text<'a, 'b, 'c, 'd> {
    android: AndroidText<'a, 'b, 'c>,
    /// The description of the authentication prompt on Apple devices.
    ///
    /// Appears as: "$(binary_name) is trying to $(description)".
    apple: &'d str,
}

pub struct AndroidText<'a, 'b, 'c> {
    title: &'a str,
    subtitle: Option<&'b str>,
    description: Option<&'c str>,
}
