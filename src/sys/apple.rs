use std::mem::MaybeUninit;

use block2::ConcreteBlock;
use icrate::{
    Foundation::{NSError, NSString},
    LocalAuthentication::{
        LAContext, LAErrorAppCancel, LAErrorAuthenticationFailed, LAErrorBiometryDisconnected,
        LAErrorBiometryLockout, LAErrorBiometryNotAvailable, LAErrorBiometryNotEnrolled,
        LAErrorBiometryNotPaired, LAErrorInvalidContext, LAErrorInvalidDimensions,
        LAErrorNotInteractive, LAErrorPasscodeNotSet, LAErrorSystemCancel, LAErrorUserCancel,
        LAErrorUserFallback, LAErrorWatchNotAvailable, LAPolicy, LAPolicyDeviceOwnerAuthentication,
        LAPolicyDeviceOwnerAuthenticationWithBiometrics,
        LAPolicyDeviceOwnerAuthenticationWithBiometricsOrWatch,
        LAPolicyDeviceOwnerAuthenticationWithWatch,
    },
};
use tokio::sync::oneshot;

use crate::{BiometricStrength, Error, Result};

#[derive(Debug)]
pub(crate) struct PolicyBuilder {
    biometrics: bool,
    password: bool,
    watch: bool,
    // wrist_detection: bool,
}

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self {
            biometrics: false,
            password: false,
            watch: false,
            // wrist_detection: false,
        }
    }

    pub(crate) const fn biometrics(self, strength: Option<BiometricStrength>) -> Self {
        Self {
            biometrics: strength.is_some(),
            ..self
        }
    }

    pub(crate) const fn password(self, password: bool) -> Self {
        Self { password, ..self }
    }

    pub(crate) const fn watch(self, watch: bool) -> Self {
        Self { watch, ..self }
    }

    // pub(crate) const fn wrist_detection(self, wrist_detection: bool) -> Self {
    //     Self {
    //         wrist_detection,
    //         ..self
    //     }
    // }

    pub(crate) const fn build(self) -> Option<Policy> {
        let policy = match (self.biometrics, self.password, self.watch) {
            (true, true, true) => LAPolicyDeviceOwnerAuthentication,
            (true, false, true) => LAPolicyDeviceOwnerAuthenticationWithBiometricsOrWatch,
            (true, false, false) => LAPolicyDeviceOwnerAuthenticationWithBiometrics,
            (false, false, true) => LAPolicyDeviceOwnerAuthenticationWithWatch,
            _ => return None,
        };
        Some(Policy { inner: policy })
    }
}

pub struct Policy {
    inner: LAPolicy,
}

pub(crate) async fn authenticate(message: &str, policy: &Policy) -> Result<()> {
    // The callback should always execute and hence a message will always be sent.
    authenticate_inner(message, policy).await.unwrap()
}

pub(crate) fn blocking_authenticate(message: &str, policy: &Policy) -> Result<()> {
    // The callback should always execute and hence a message will always be sent.
    authenticate_inner(message, policy).blocking_recv().unwrap()
}

fn authenticate_inner(message: &str, policy: &Policy) -> oneshot::Receiver<Result<()>> {
    let (tx, rx) = oneshot::channel();
    let unsafe_tx = MaybeUninit::new(tx);

    let block = ConcreteBlock::new(move |is_success, error: *mut NSError| {
        // SAFETY: The callback is only executed once.
        let tx = unsafe { unsafe_tx.assume_init_read() };
        let _ = if bool::from(is_success) {
            tx.send(Ok(()))
        } else {
            let code = unsafe { &*error }.code();
            #[allow(non_upper_case_globals)]
            let error = match code {
                LAErrorAppCancel => Error::AppCanceled,
                LAErrorAuthenticationFailed => Error::Authentication,
                LAErrorBiometryDisconnected => Error::BiometryDisconnected,
                LAErrorBiometryLockout => Error::Exhausted,
                // NOTE: This is triggered when access to biometrics is denied.
                LAErrorBiometryNotAvailable => Error::Unavailable,
                LAErrorBiometryNotEnrolled => Error::NotEnrolled,
                LAErrorBiometryNotPaired => Error::NotPaired,
                // This error shouldn't occur, because we never invalidate the context.
                LAErrorInvalidContext => Error::Unknown,
                LAErrorInvalidDimensions => Error::InvalidDimensions,
                LAErrorNotInteractive => Error::NotInteractive,
                LAErrorPasscodeNotSet => Error::PasscodeNotSet,
                LAErrorSystemCancel => Error::SystemCanceled,
                LAErrorUserCancel => Error::UserCanceled,
                // TODO
                LAErrorUserFallback => Error::Unknown,
                LAErrorWatchNotAvailable => Error::WatchNotAvailable,
                _ => Error::Unknown,
            };
            tx.send(Err(error))
        };
    })
    .copy();

    let context = unsafe { LAContext::new() };

    unsafe {
        context.evaluatePolicy_localizedReason_reply(
            policy.inner,
            &NSString::from_str(message),
            &block,
        )
    };

    rx
}
