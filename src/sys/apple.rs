use std::mem::MaybeUninit;
#[cfg(not(feature = "async"))]
use std::sync::mpsc as channel_impl;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LAContext, LAError, LAPolicy};
#[cfg(feature = "async")]
use tokio::sync::oneshot as channel_impl;

use crate::{BiometricStrength, Error, Result, Text};

pub(crate) type RawContext = ();

#[derive(Debug)]
pub(crate) struct Context {
    inner: Retained<LAContext>,
}

impl Context {
    pub(crate) fn new(_: RawContext) -> Self {
        Self {
            inner: unsafe { LAContext::new() },
        }
    }

    #[cfg(feature = "async")]
    pub(crate) async fn authenticate(
        &self,
        text: Text<'_, '_, '_, '_, '_>,
        policy: &Policy,
    ) -> Result<()> {
        // The callback should always execute and hence a message will always be sent.
        self.authenticate_inner(text, policy).await.unwrap()
    }

    pub(crate) fn blocking_authenticate(&self, text: Text, policy: &Policy) -> Result<()> {
        // The callback should always execute, hence a message will always be sent, and
        // hence it is ok to unwrap.
        #[cfg(feature = "async")]
        {
            self.authenticate_inner(text, policy)
                .blocking_recv()
                .expect("failed to receive message from authentication callback")
        }
        #[cfg(not(feature = "async"))]
        {
            self.authenticate_inner(text, policy)
                .recv()
                .expect("failed to receive message from authentication callback")
        }
    }

    fn authenticate_inner(
        &self,
        text: Text<'_, '_, '_, '_, '_>,
        policy: &Policy,
    ) -> channel_impl::Receiver<Result<()>> {
        let (tx, rx) = channel_impl::channel();
        let unsafe_tx = MaybeUninit::new(tx);
        let message = text.apple;

        let block = RcBlock::new(move |is_success, error: *mut NSError| {
            // SAFETY: The callback is only executed once.
            let tx = unsafe { unsafe_tx.assume_init_read() };
            let _ = if bool::from(is_success) {
                tx.send(Ok(()))
            } else {
                let code = unsafe { &*error }.code();
                #[allow(non_upper_case_globals)]
                let error = match LAError(code) {
                    LAError::AppCancel => Error::AppCanceled,
                    LAError::AuthenticationFailed => Error::Authentication,
                    LAError::BiometryDisconnected => Error::BiometryDisconnected,
                    LAError::BiometryLockout => Error::Exhausted,
                    // NOTE: This is triggered when access to biometrics is denied.
                    LAError::BiometryNotAvailable => Error::Unavailable,
                    LAError::BiometryNotEnrolled => Error::NotEnrolled,
                    LAError::BiometryNotPaired => Error::NotPaired,
                    // This error shouldn't occur, because we never invalidate the context.
                    LAError::InvalidContext => Error::Unknown,
                    LAError::InvalidDimensions => Error::InvalidDimensions,
                    LAError::NotInteractive => Error::NotInteractive,
                    LAError::PasscodeNotSet => Error::PasscodeNotSet,
                    LAError::SystemCancel => Error::SystemCanceled,
                    LAError::UserCancel => Error::UserCanceled,
                    // TODO
                    LAError::UserFallback => Error::Unknown,
                    LAError::WatchNotAvailable => Error::WatchNotAvailable,
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

#[derive(Debug)]
pub(crate) struct Policy {
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
            _biometrics: true,
            _password: true,
            _watch: true,
            _wrist_detection: true,
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
        // TODO: Test watchos

        #[cfg(target_os = "watchos")]
        let policy = match self {
            Self {
                _password: true,
                _wrist_detection: true,
                ..
            } => LAPolicy::DeviceOwnerAuthenticationWithWristDetection,
            Self {
                _password: true,
                _wrist_detection: false,
                ..
            } => LAPolicy::DeviceOwnerAuthentication,
            _ => return None,
        };

        #[cfg(not(target_os = "watchos"))]
        let policy = match self {
            Self {
                _biometrics: true,
                _password: true,
                _watch: true,
                ..
            } => LAPolicy::DeviceOwnerAuthentication,
            Self {
                _biometrics: true,
                _password: false,
                _watch: true,
                ..
            } => LAPolicy::DeviceOwnerAuthenticationWithBiometricsOrWatch,
            Self {
                _biometrics: true,
                _password: false,
                _watch: false,
                ..
            } => LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
            Self {
                _biometrics: false,
                _password: false,
                _watch: true,
                ..
            } => LAPolicy::DeviceOwnerAuthenticationWithWatch,
            _ => return None,
        };
        Some(Policy { inner: policy })
    }
}
