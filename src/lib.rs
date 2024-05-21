//! A cross-platform authentication framework.
//!
//! This crate supports:
//! - Apple. More specifically, it uses the [`LAContext`] API, so it supports
//!   all OS versions that support [`LAContext`].
//! - Windows
//! - Linux ([`polkit`]). Still a work in progress.
//! - Android
//!
//! # Android
//!
//!
//! [`LAContext`]: https://developer.apple.com/documentation/localauthentication/lacontext
//! [`polkit`]: https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html

mod error;
mod sys;

pub use crate::error::{Error, Result};

pub type RawContext = sys::RawContext;

#[derive(Debug)]
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

    /// Authenticates using the provided policy and message.
    ///
    /// Returns whether the authentication was successful.
    #[inline]
    pub async fn authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        self.inner.authenticate(message, &policy.inner).await
    }

    /// Authenticates using the provided policy and message.
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
    /// Returns a new policy with sane defaults.
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
    /// ```no_run
    /// #![feature(const_option)]
    ///
    /// use robius_authentication::{BiometricStrength, Context, Policy, PolicyBuilder};
    ///
    /// const POLICY: Policy = PolicyBuilder::new()
    ///     .biometrics(Some(BiometricStrength::Strong))
    ///     .build()
    ///     .expect("invalid context configuration");
    ///
    /// Context::new(())
    ///     .blocking_authenticate("something", &POLICY)
    ///     .expect("authentication failed");
    /// ```
    #[inline]
    #[must_use]
    pub const fn biometrics(self, strength: Option<BiometricStrength>) -> Self {
        Self {
            inner: self.inner.biometrics(strength),
        }
    }

    /// Sets whether the policy supports passwords.
    #[inline]
    #[must_use]
    pub const fn password(self, password: bool) -> Self {
        Self {
            inner: self.inner.password(password),
        }
    }

    /// Sets whether the policy supports watch proximity authentication.
    ///
    /// This only has an effect on iOS and macOS.
    #[inline]
    #[must_use]
    pub const fn watch(self, watch: bool) -> Self {
        Self {
            inner: self.inner.watch(watch),
        }
    }

    /// Sets whether the policy requires the watch to be on the user's wrist.
    ///
    /// This only has an effect on watchOS.
    #[inline]
    #[must_use]
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
    #[must_use]
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
#[derive(Debug)]
pub struct Policy {
    inner: sys::Policy,
}
