// #![feature(const_option)]

use robius_authentication::{AndroidText, BiometricStrength, Context, PolicyBuilder, Text};

fn main() {
    let policy = PolicyBuilder::new()
        .biometrics(Some(BiometricStrength::Strong))
        .password(true)
        .watch(true)
        .build()
        .unwrap();

    let context = Context::new(());

    if context
        .blocking_authenticate(
            Text {
                android: AndroidText {
                    title: "Title",
                    subtitle: None,
                    description: None,
                },
                apple: "authenticate",
                windows: "authenticate",
            },
            &policy,
        )
        .is_ok()
    {
        println!("Authorized");
    } else {
        println!("Unauthorized");
    }
}
