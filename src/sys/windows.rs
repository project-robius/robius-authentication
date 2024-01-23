use windows::{
    core::HSTRING,
    Foundation::IAsyncOperation,
    Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier},
};

use crate::BiometricStrength;

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

    // pub(crate) const fn wrist_detection(self, _: bool) -> Self {
    //  Self
    // }

    pub(crate) const fn build(self) -> Option<Policy> {
        Some(Policy)
    }
}

pub(crate) struct Policy;

pub(crate) struct Context;

impl Context {
    pub(crate) fn new() -> Self {
        Self
    }

    fn authenticate_inner(message: &str) -> IAsyncOperation<UserConsentVerificationResult> {
        // TODO: len
        let mut caption = Vec::with_capacity(message.len());

        for c in message.encode_utf16() {
            caption.push(c);
        }
        caption.push(0);

        UserConsentVerifier::RequestVerificationAsync(&HSTRING::from_wide(&caption[..]).unwrap())
            .unwrap()
    }

    pub(crate) async fn authenticate(&self, message: &str, _: &Policy) -> bool {
        Self::authenticate_inner(message).await.unwrap() == UserConsentVerificationResult::Verified
    }

    pub(crate) fn blocking_authenticate(&self, message: &str, _: &Policy) -> bool {
        todo!();
    }
}
