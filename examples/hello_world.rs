#![feature(const_option)]

use robius_authentication::{
    AndroidText, BiometricStrength, Context, Policy, PolicyBuilder, Text, WindowsText,
};

const POLICY: Policy = PolicyBuilder::new()
    .biometrics(Some(BiometricStrength::Strong))
    .password(true)
    .watch(true)
    .build()
    .unwrap();

const TEXT: Text = Text {
    android: AndroidText {
        title: "Title",
        subtitle: None,
        description: None,
    },
    apple: "authenticate",
    windows: WindowsText::new("Title", "Description").unwrap(),
};

fn main() {
    let context = Context::new(());

    if context.blocking_authenticate(TEXT, &POLICY).is_ok() {
        println!("Authorized");
    } else {
        println!("Unauthorized");
    }
}
