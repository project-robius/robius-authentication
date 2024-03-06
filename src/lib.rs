//! A cross-platform authentication framework.
//!
//! This crate supports:
//! - Apple. More specifically, it uses the [`LAContext`] API, so it supports
//!   all OS versions that support [`LAContext`].
//! - Windows.
//!
//! [`LAContext`]: https://developer.apple.com/documentation/localauthentication/lacontext

mod error;
mod sys;

pub use error::{Error, Result};
use jni::objects::JObject;
#[cfg(target_os = "android")]
use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};

/// A biometric strength class.
///
/// This only has an effect on Android. On other targets, any biometric strength
/// setting will enable all biometric authentication devices. See the [Android
/// documentation][android-docs] for more details.
///
/// [android-docs]: https://source.android.com/docs/security/features/biometric
pub enum BiometricStrength {
    Strong,
    Weak,
    Convenience,
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
    /// use robius_authentication::{blocking_authenticate, BiometricStrength, Policy, PolicyBuilder};
    ///
    /// const POLICY: Policy = PolicyBuilder::new()
    ///     .biometrics(Some(BiometricStrength::Strong))
    ///     .build()
    ///     .expect("invalid context configuration");
    ///
    /// // Authenticates with biometrics.
    /// blocking_authenticate("login", &POLICY)?;
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
///
/// # Usage
pub struct Policy {
    inner: sys::Policy,
}

/// Asynchronously authenticate a policy.
///
/// Returns whether the authentication was successful.
#[inline]
pub async fn authenticate(message: &str, policy: &Policy) -> Result<()> {
    sys::authenticate(message, &policy.inner).await
}

#[inline]
pub fn blocking_authenticate(ctx: JObject, message: &str, policy: &Policy) -> Result<()> {
    sys::blocking_authenticate(ctx, message, &policy.inner)
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_robius_authentication_AuthenticationCallback_rustCallback<'a>(
    mut env: JNIEnv<'a>,
    _: JObject<'a>,
) {
    log::error!("HECLRUHRLOEUHROCLEUH TESTING");
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_com_example_myapplication2_Test_greeting<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    input: JObject<'a>,
) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Error)
            .with_tag("mytag"), // logs will show under mytag tag
    );

    let policy = PolicyBuilder::new().build().unwrap();
    let _ = blocking_authenticate(input, "something", &policy);
}
