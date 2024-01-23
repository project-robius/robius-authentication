use std::mem::MaybeUninit;

use block2::ConcreteBlock;
use icrate::{
    objc2::rc::Id,
    Foundation::{ns_string, NSString},
    LocalAuthentication::{
        LAContext, LAPolicy, LAPolicyDeviceOwnerAuthentication,
        LAPolicyDeviceOwnerAuthenticationWithBiometrics,
        LAPolicyDeviceOwnerAuthenticationWithBiometricsOrWatch,
        LAPolicyDeviceOwnerAuthenticationWithWatch,
    },
};
use tokio::sync::oneshot;

use crate::BiometricStrength;

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

pub struct Context {
    inner: Id<LAContext>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            inner: unsafe { LAContext::new() },
        }
    }

    fn authenticate_inner(&self, message: &str, policy: &Policy) -> oneshot::Receiver<bool> {
        unsafe { self.inner.canEvaluatePolicy_error(policy.inner) }.unwrap();

        let (tx, rx) = oneshot::channel();
        let unsafe_tx = MaybeUninit::new(tx);

        let block = ConcreteBlock::new(move |is_success, _error| {
            // SAFETY: The callback is only executed once.
            // NOTE: This may not succeed if the receiving future is dropped, but we don't
            // really care.
            let _ = unsafe { unsafe_tx.assume_init_read() }.send(bool::from(is_success));
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

    pub(crate) async fn authenticate(&self, message: &str, policy: &Policy) -> bool {
        // The callback should always execute and hence a message will always be sent.
        self.authenticate_inner(message, policy).await.unwrap()
    }

    pub(crate) fn blocking_authenticate(&self, message: &str, policy: &Policy) -> bool {
        // The callback should always execute and hence a message will always be sent.
        self.authenticate_inner(message, policy)
            .blocking_recv()
            .unwrap()
    }
}
