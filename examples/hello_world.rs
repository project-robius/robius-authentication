// #![feature(const_option)]

use robius_authentication::{BiometricStrength, Context, PolicyBuilder};

fn main() {
    let policy = PolicyBuilder::new()
        .biometrics(Some(BiometricStrength::Strong))
        .password(true)
        .watch(true)
        .build()
        .unwrap();

    let context = Context::new(());

    if context
        .blocking_authenticate("verify your identity", &policy)
        .is_ok()
    {
        println!("Authorized");
    } else {
        println!("Unauthorized");
    }
}
