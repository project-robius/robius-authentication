#![feature(const_option)]

use robius_authentication::{blocking_authenticate, BiometricStrength, Policy, PolicyBuilder};

fn main() {
    const POLICY: Policy = PolicyBuilder::new()
        .biometrics(Some(BiometricStrength::Strong))
        .password(true)
        .watch(true)
        .build()
        .unwrap();

    if blocking_authenticate("verify your identity", &POLICY).is_ok() {
        println!("Authorized");
    } else {
        println!("Unauthorized");
    }
}
