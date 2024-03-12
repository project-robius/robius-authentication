use std::mem::MaybeUninit;

use block2::ConcreteBlock;
use icrate::{
    Foundation::{NSError, NSString},
    LocalAuthentication::{
        LAContext, LAErrorAppCancel, LAErrorAuthenticationFailed, LAErrorBiometryDisconnected,
        LAErrorBiometryLockout, LAErrorBiometryNotAvailable, LAErrorBiometryNotEnrolled,
        LAErrorBiometryNotPaired, LAErrorInvalidContext, LAErrorInvalidDimensions,
        LAErrorNotInteractive, LAErrorPasscodeNotSet, LAErrorSystemCancel, LAErrorUserCancel,
        LAErrorUserFallback, LAErrorWatchNotAvailable, LAPolicy,
    },
};
use objc2::rc::Id;
use tokio::sync::oneshot;

use crate::{BiometricStrength, Error, Result};

pub(crate) type RawContext = ();

pub(crate) struct Context {
    inner: Id<LAContext>,
}

impl Context {
    pub(crate) fn new(_: RawContext) -> Self {
        Self {
            inner: unsafe { LAContext::new() },
        }
    }

    pub(crate) async fn authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        // The callback should always execute and hence a message will always be sent.
        self.authenticate_inner(message, policy).await.unwrap()
    }

    pub(crate) fn blocking_authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        // The callback should always execute and hence a message will always be sent.
        self.authenticate_inner(message, policy)
            .blocking_recv()
            .unwrap()
    }

    fn authenticate_inner(&self, message: &str, policy: &Policy) -> oneshot::Receiver<Result<()>> {
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

        unsafe {
            self.inner.evaluatePolicy_localizedReason_reply(
                policy.inner,
                &NSString::from_str(message),
                &block,
            )
        };

        rx
    }
}

pub struct Policy {
    inner: LAPolicy,
}

#[derive(Debug)]
pub(crate) struct PolicyBuilder {
    _biometrics: bool,
    _password: bool,
    _watch: bool,
    _wrist_detection: bool,
}

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self {
            _biometrics: false,
            _password: false,
            _watch: false,
            _wrist_detection: false,
        }
    }

    pub(crate) const fn biometrics(self, strength: Option<BiometricStrength>) -> Self {
        Self {
            _biometrics: strength.is_some(),
            ..self
        }
    }

    pub(crate) const fn password(self, password: bool) -> Self {
        Self {
            _password: password,
            ..self
        }
    }

    pub(crate) const fn watch(self, watch: bool) -> Self {
        Self {
            _watch: watch,
            ..self
        }
    }

    pub(crate) const fn wrist_detection(self, wrist_detection: bool) -> Self {
        Self {
            _wrist_detection: wrist_detection,
            ..self
        }
    }

    pub(crate) const fn build(self) -> Option<Policy> {
        use icrate::LocalAuthentication as la;

        // TODO: Test

        #[cfg(target_os = "watchos")]
        let policy = match self {
            Self {
                _password: true,
                _wrist_detection: true,
                ..
            } => la::LAPolicyDeviceOwnerAuthenticationWithWristDetection,
            Self {
                _password: true,
                _wrist_detection: false,
                ..
            } => la::LAPolicyDeviceOwnerAuthentication,
            _ => return None,
        };

        #[cfg(not(target_os = "watchos"))]
        let policy = match self {
            Self {
                _biometrics: true,
                _password: true,
                _watch: true,
                ..
            } => la::LAPolicyDeviceOwnerAuthentication,
            Self {
                _biometrics: true,
                _password: false,
                _watch: true,
                ..
            } => la::LAPolicyDeviceOwnerAuthenticationWithBiometricsOrWatch,
            Self {
                _biometrics: true,
                _password: false,
                _watch: false,
                ..
            } => la::LAPolicyDeviceOwnerAuthenticationWithBiometrics,
            Self {
                _biometrics: false,
                _password: false,
                _watch: true,
                ..
            } => la::LAPolicyDeviceOwnerAuthenticationWithWatch,
            _ => return None,
        };
        Some(Policy { inner: policy })
    }
}
