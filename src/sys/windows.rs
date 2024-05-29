use windows::{
    core::{factory, HSTRING},
    Foundation::IAsyncOperation,
    Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier},
    Win32::{
        System::WinRT::IUserConsentVerifierInterop, UI::WindowsAndMessaging::GetDesktopWindow,
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
        text: Text<'_, '_, '_, '_, '_>,
        _: &Policy,
    ) -> Result<()> {
        convert(request_verification(text.windows)?.await?)
        // TODO: Fallback to password if unavailable?
        // https://github.com/tsoutsman/robius-authentication/blob/ddb08e75c452ece39ae9b807c7aeb21161836332/src/sys/windows.rs
    }

    pub(crate) fn blocking_authenticate(&self, text: Text, _: &Policy) -> Result<()> {
        convert(request_verification(text.windows)?.get()?)
        // TODO: Fallback to password?
        // https://github.com/tsoutsman/robius-authentication/blob/ddb08e75c452ece39ae9b807c7aeb21161836332/src/sys/windows.rs
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

fn request_verification(message: &str) -> Result<IAsyncOperation<UserConsentVerificationResult>> {
    let window = unsafe { GetDesktopWindow() };
    let caption = caption(message);

    let factory = factory::<UserConsentVerifier, IUserConsentVerifierInterop>()?;

    unsafe {
        IUserConsentVerifierInterop::RequestVerificationForWindowAsync(
            &factory,
            window,
            &HSTRING::from_wide(&caption[..])?,
        )
    }
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
