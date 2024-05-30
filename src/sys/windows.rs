use windows::{
    core::HSTRING,
    Foundation::IAsyncOperation,
    Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    },
};

use crate::{BiometricStrength, Error, Result, Text};

pub(crate) type RawContext = ();

#[derive(Debug)]
pub(crate) struct Context;

impl Context {
    pub(crate) fn new(_: RawContext) -> Self {
        Self
    }

    #[cfg(feature = "async")]
    pub(crate) async fn authenticate(
        &self,
        message: Text<'_, '_, '_, '_, '_>,
        _: &Policy,
    ) -> Result<()> {
        // NOTE: If we don't check availability, `request_verification` will hang.

        if check_availability()?.await == Ok(UserConsentVerifierAvailability::Available) {
            convert(request_verification(message.windows)?.await?)
        } else {
            // TODO: Fallback to password?
            // https://github.com/tsoutsman/robius-authentication/blob/ddb08e75c452ece39ae9b807c7aeb21161836332/src/sys/windows.rs
            Err(Error::Unavailable)
        }
    }

    pub(crate) fn blocking_authenticate(&self, message: Text, _: &Policy) -> Result<()> {
        // NOTE: If we don't check availability, `request_verification` will hang.

        if check_availability()?.get() == Ok(UserConsentVerifierAvailability::Available) {
            convert(request_verification(message.windows)?.get()?)
        } else {
            // TODO: Fallback to password?
            // https://github.com/tsoutsman/robius-authentication/blob/ddb08e75c452ece39ae9b807c7aeb21161836332/src/sys/windows.rs
            Err(Error::Unavailable)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Policy;

#[derive(Debug)]
pub(crate) struct PolicyBuilder {
    valid: bool,
}

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self { valid: true }
    }

    pub(crate) const fn biometrics(self, biometrics: Option<BiometricStrength>) -> Self {
        if biometrics.is_none() {
            Self { valid: false }
        } else {
            self
        }
    }

    pub(crate) const fn password(self, password: bool) -> Self {
        if password {
            self
        } else {
            Self { valid: false }
        }
    }

    pub(crate) const fn watch(self, _: bool) -> Self {
        self
    }

    pub(crate) const fn wrist_detection(self, _: bool) -> Self {
        self
    }

    pub(crate) const fn build(self) -> Option<Policy> {
        if self.valid {
            Some(Policy)
        } else {
            None
        }
    }
}

fn check_availability() -> Result<IAsyncOperation<UserConsentVerifierAvailability>> {
    UserConsentVerifier::CheckAvailabilityAsync().map_err(|e| e.into())
}

fn request_verification(message: &str) -> Result<IAsyncOperation<UserConsentVerificationResult>> {
    let caption = caption(message);

    UserConsentVerifier::RequestVerificationAsync(&HSTRING::from_wide(&caption[..])?)
        .map_err(|e| e.into())
}

fn caption(message: &str) -> Vec<u16> {
    let mut caption = Vec::with_capacity(message.len());

    for c in message.encode_utf16() {
        caption.push(c);
    }
    caption.push(0);

    caption
}

fn convert(result: UserConsentVerificationResult) -> Result<()> {
    match result {
        UserConsentVerificationResult::Verified => Ok(()),
        UserConsentVerificationResult::DeviceNotPresent => Err(Error::Unavailable),
        UserConsentVerificationResult::NotConfiguredForUser => Err(Error::Unavailable),
        UserConsentVerificationResult::DisabledByPolicy => Err(Error::Unavailable),
        UserConsentVerificationResult::DeviceBusy => Err(Error::Busy),
        UserConsentVerificationResult::RetriesExhausted => Err(Error::Exhausted),
        UserConsentVerificationResult::Canceled => Err(Error::UserCanceled),
        _ => Err(Error::Unknown),
    }
}

impl From<windows::core::Error> for Error {
    fn from(_value: windows::core::Error) -> Self {
        // TODO
        // match value.code().0 {
        //     _ => Self::Unknown,
        // }
        Self::Unknown
    }
}
