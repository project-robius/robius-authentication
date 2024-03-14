//! A cross-platform authentication framework.
//!
//! This crate supports:
//! - Apple. More specifically, it uses the [`LAContext`] API, so it supports
//!   all OS versions that support [`LAContext`].
//! - Windows
//! - Linux (polkit)
//! - Android
//!
//! [`LAContext`]: https://developer.apple.com/documentation/localauthentication/lacontext

mod error;
mod sys;

pub use crate::error::{Error, Result};

pub type RawContext = sys::RawContext;

pub struct Context {
    inner: sys::Context,
}

impl Context {
    #[inline]
    pub fn new(raw: RawContext) -> Self {
        Self {
            inner: sys::Context::new(raw),
        }
    }

    /// Asynchronously authenticate a policy.
    ///
    /// Returns whether the authentication was successful.
    #[inline]
    pub async fn authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        self.inner.authenticate(message, &policy.inner).await
    }

    /// Authenticate a policy, blocking until it completes (in a non-async
    /// context).
    ///
    /// Returns whether the authentication was successful.
    #[inline]
    pub fn blocking_authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        self.inner.blocking_authenticate(message, &policy.inner)
    }
}

/// A biometric strength class.
///
/// This only has an effect on Android. On other targets, any biometric strength
/// setting will enable all biometric authentication devices. See the [Android
/// documentation][android-docs] for more details.
///
/// [android-docs]: https://source.android.com/docs/security/features/biometric
#[derive(Debug)]
pub enum BiometricStrength {
    Strong,
    Weak,
}

#[derive(Debug)]
pub struct PolicyBuilder {
    inner: sys::PolicyBuilder,
}

impl Default for PolicyBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyBuilder {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: sys::PolicyBuilder::new(),
        }
    }

    /// Configures biometric authentication with the given strength.
    ///
    /// The strength only has an effect on Android, see [`BiometricStrength`]
    /// for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(const_option)]
    ///
    /// use robius_authentication::{BiometricStrength, Context, Policy, PolicyBuilder};
    ///
    /// const POLICY: Policy = PolicyBuilder::new()
    ///     .biometrics(Some(BiometricStrength::Strong))
    ///     .build()
    ///     .expect("invalid context configuration");
    ///
    /// Context::new(()).authenticate("something", &POLICY).unwrap();
    /// ```
    #[inline]
    pub const fn biometrics(self, strength: Option<BiometricStrength>) -> Self {
        Self {
            inner: self.inner.biometrics(strength),
        }
    }

    /// Sets whether the policy supports passwords.
    #[inline]
    pub const fn password(self, password: bool) -> Self {
        Self {
            inner: self.inner.password(password),
        }
    }

    /// Sets whether the policy supports watch proximity authentication.
    ///
    /// This only has an effect on iOS and macOS.
    #[inline]
    pub const fn watch(self, watch: bool) -> Self {
        Self {
            inner: self.inner.watch(watch),
        }
    }

    /// Sets whether the policy requires the watch to be on the user's wrist.
    ///
    /// This only has an effect on watchOS.
    #[inline]
    pub const fn wrist_detection(self, wrist_detection: bool) -> Self {
        Self {
            inner: self.inner.wrist_detection(wrist_detection),
        }
    }

    /// Constructs the policy.
    ///
    /// Returns `None` if the specified configuration is not valid for the
    /// current target.
    #[inline]
    pub const fn build(self) -> Option<Policy> {
        Some(Policy {
            // TODO: feature(const_try)
            inner: match self.inner.build() {
                Some(inner) => inner,
                None => return None,
            },
        })
    }
}

/// An authentication policy.
pub struct Policy {
    inner: sys::Policy,
}

// TODO: This is currently a hack so that an application crate doesn't need to
// sync `jni` crate versions with `robius_authentication`. In future, there will
// be a better solution.

#[cfg(target_os = "android")]
pub mod jni {
    pub use jni::{
        objects::{GlobalRef, JObject},
        JavaVM,
    };

    pub enum ActivityObject<'j> {
        JObject(JObject<'j>),
        GlobalRef(GlobalRef),
    }
}
