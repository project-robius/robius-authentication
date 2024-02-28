use crate::{BiometricStrength, Result};

pub struct Policy;

#[derive(Debug)]
pub(crate) struct PolicyBuilder;

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) const fn biometrics(self, _: Option<BiometricStrength>) -> Self {
        Self
    }

    pub(crate) const fn password(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn watch(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn wrist_detection(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn build(self) -> Option<Policy> {
        Some(Policy)
    }
}

pub(crate) async fn authenticate(_message: &str, _: &Policy) -> Result<()> {
    unimplemented!()
}

pub(crate) fn blocking_authenticate(_message: &str, _: &Policy) -> Result<()> {
    // use std::{os::unix::process::CommandExt, process::Command};

    // use pam_client::conv_mock::Conversation; // CLI implementation
    // use pam_client::{Context, Flag};
    //
    // let mut context = Context::new(
    //     "my-service", // Service name, decides which policy is used (see
    // `/etc/pam.d`)     None,         // Optional preset username
    //     Conversation::with_credentials("klim", "abcd"), // Handler for user
    // interaction )
    // .expect("Failed to initialize PAM context");
    //
    // Optionally set some settings
    // context.set_user_prompt(Some("Who art thou? ")).unwrap();
    //
    // Authenticate the user (ask for password, 2nd-factor token, fingerprint,
    // etc.) context
    //     .authenticate(Flag::NONE)
    //     .expect("Authentication failed");
    //
    // Validate the account (is not locked, expired, etc.)
    // context
    //     .acct_mgmt(Flag::NONE)
    //     .expect("Account validation failed");
    //
    // let username = context.user();
    //
    // Open session and initialize credentials
    // let mut session = context
    //     .open_session(Flag::NONE)
    //     .expect("Session opening failed");
    //
    // The session is automatically closed when it goes out of scope.
    // unimplemented!("hello")
    use pam::Client;

    let mut client = Client::with_password("abcd").expect("Failed to init PAM client.");
    // Preset the login & password we will use for authentication
    client.conversation_mut().set_credentials("klim", "abcd");
    // Actually try to authenticate:
    client.authenticate().expect("Authentication failed!");
    // Now that we are authenticated, it's possible to open a sesssion:
    client.open_session().expect("Failed to open a session!");
    Ok(())
}
