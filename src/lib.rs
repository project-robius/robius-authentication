mod sys;

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
    /// use robius_authentication::{BiometricStrength, Policy, PolicyBuilder};
    ///
    /// const POLICY: Policy = PolicyBuilder::new()
    ///     .biometrics(Some(BiometricStrength::Strong))
    ///     .build()
    ///     .expect("invalid context configuration");
    ///
    /// // Authenticates with biometrics.
    /// // context.authenticate_sync();
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

/// An authentication context.
///
/// A program should have one global authentication context, that can be used to
/// authenticate different policies.
///
/// # Usage
///
/// ```
/// #![feature(const_option)]
///
/// use robius_authentication::{BiometricStrength, Context, Policy, PolicyBuilder};
///
/// const POLICY_A: Policy = PolicyBuilder::new()
///     .biometrics(Some(BiometricStrength::Strong))
///     .password(false)
///     .watch(false)
///     .build()
///     .unwrap();
///
/// const POLICY_B: Policy = PolicyBuilder::new()
///     .biometrics(Some(BiometricStrength::Strong))
///     .password(true)
///     .watch(true)
///     .build()
///     .unwrap();
///
/// let context = Context::new();
///
/// context.blocking_authenticate(&POLICY_A);
/// context.blocking_authenticate(&POLICY_B);
/// ```
pub struct Context {
    inner: sys::Context,
}

impl Default for Context {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: sys::Context::new(),
        }
    }

    /// Asynchronously authenticate a policy.
    ///
    /// Returns whether the authentication was successful.
    #[inline]
    pub async fn authenticate(&self, message: &str, policy: &Policy) -> bool {
        self.inner.authenticate(message, &policy.inner).await
    }

    #[inline]
    pub fn blocking_authenticate(&self, message: &str, policy: &Policy) -> bool {
        self.inner.blocking_authenticate(message, &policy.inner)
    }
}
