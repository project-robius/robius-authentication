/// The result of an authentication operation.
pub type Result<T> = std::result::Result<T, Error>;

// TODO: How specific do we want the errors to be?

/// An error produced during authentication.
#[derive(Debug)]
pub enum Error {
    // TODO: Reexport jni::errors::Error
    Java(jni::errors::Error),

    // Common errors
    /// The user failed to provide valid credentials.
    ///
    /// This error can occur on:
    /// - [Apple]
    /// - [Windows]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorauthenticationfailed
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    Authentication,
    /// Authentication failed because there were too many failed attempts.
    ///
    /// This error can occur on:
    /// - [Apple]
    /// - [Windows]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorbiometrylockout
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    #[doc(alias = "lockout")]
    Exhausted,
    /// The requested authentication method was unavailable.
    ///
    /// This error can occur on:
    /// - [Apple]
    /// - [Windows]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorbiometrynotavailable
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    Unavailable,
    /// The user canceled authentication.
    ///
    /// This error can occur on:
    /// - [Apple]
    /// - [Windows]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorusercancel
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    UserCanceled,

    // Apple-specific errors
    /// The app canceled authentication.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorappcancel
    AppCanceled,
    /// The system canceled authentication.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorsystemcancel
    SystemCanceled,
    /// The device supports biometry only using a removable accessory, but the
    /// paired accessory isn’t connected.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorbiometrydisconnected
    BiometryDisconnected,
    /// The device supports biometry only using a removable accessory, but no
    /// accessory is paired.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorbiometrynotpaired
    NotPaired,
    /// The user has no enrolled biometric identities.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorbiometrynotenrolled
    NotEnrolled,
    /// Displaying the required authentication user interface is forbidden.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrornotinteractive
    NotInteractive,
    /// An attempt to authenticate with Apple Watch failed.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorwatchnotavailable
    WatchNotAvailable,
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorinvaliddimensions
    InvalidDimensions,
    /// A passcode isn’t set on the device.
    ///
    /// This error can occur on:
    /// - [Apple]
    ///
    /// [Apple]: https://developer.apple.com/documentation/localauthentication/laerror/laerrorpasscodenotset
    PasscodeNotSet,

    // Windows-specific errors
    /// The biometric verifier device is performing an operation and is
    /// unavailable.
    ///
    /// This error can occur on:
    /// - [Windows]
    ///
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    Busy,
    /// Group policy has disabled the biometric verifier device.
    ///
    /// This error can occur on:
    /// - [Windows]
    ///
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    DisabledByPolicy,
    /// A biometric verifier device is not configured for this user.
    ///
    /// This error can occur on:
    /// - [Windows]
    ///
    /// [Windows]: https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverificationresult
    NotConfigured,

    /// An unknown error occurred.
    Unknown,
}

impl From<jni::errors::Error> for Error {
    fn from(value: jni::errors::Error) -> Self {
        Self::Java(value)
    }
}
