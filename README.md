# `robius-authentication`

[![Latest Version](https://img.shields.io/crates/v/robius-authentication.svg)](https://crates.io/crates/robius_authentication)
[![Docs](https://docs.rs/robius-authentication/badge.svg)](https://docs.rs/robius-authentication/latest/robius_authentication/)
[![Project Robius Matrix Chat](https://img.shields.io/matrix/robius-general%3Amatrix.org?server_fqdn=matrix.org&style=flat&logo=matrix&label=Project%20Robius%20Matrix%20Chat&color=B7410E)](https://matrix.to/#/#robius:matrix.org)

Rust abstractions for multi-platform native authentication.

This crate supports:
* Apple: TouchID, FaceID, and regular username/password on both macOS and iOS.
* Android: See below for additional steps.
  * Requires the `USE_BIOMETRIC` permission in your app's manifest.
* Windows: Windows Hello (face recognition, fingerprint, PIN),
plus winrt-based fallback for username/password.
* Linux: [`polkit`]-based authentication using the desktop environment's prompt.
  * **Note: Linux support is currently incomplete.**

## Usage on Android

For authentication to work, the following must be added to your app's
`AndroidManifest.xml`:
```xml
<uses-permission android:name="android.permission.USE_BIOMETRIC" />
```

## Example

```no_run
use robius_authentication::{
    AndroidText, BiometricStrength, Context, Policy, PolicyBuilder, Text, WindowsText,
};

let policy: Policy = PolicyBuilder::new()
    .biometrics(Some(BiometricStrength::Strong))
    .password(true)
    .watch(true)
    .build()
    .unwrap();

let text = Text {
    android: AndroidText {
        title: "Title",
        subtitle: None,
        description: None,
    },
    apple: "authenticate",
    windows: WindowsText::new("Title", "Description"),
};

let auth_result = Context::new(()).blocking_authenticate(text, &policy);
...
```

For more details about the prompt text, see the `Text` struct,
which allows you to customize the prompt for each platform.

[`polkit`]: https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html
