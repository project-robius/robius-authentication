mod callback;

use callback::{Receiver, Sender};
use jni::{
    objects::{GlobalRef, JObject, JValueGen},
    JNIEnv,
};

use crate::{BiometricStrength, Error, Result};

pub(crate) type RawContext = ();

// Actual contextual info is handled by the `robius-android-env` crate, so we
// don't have to store any state here.
#[derive(Debug)]
pub(crate) struct Context;

impl Context {
    pub(crate) fn new(_: RawContext) -> Self {
        Self
    }

    pub(crate) async fn authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        // TODO: `result_flattening` feature
        if let Ok(inner) = self.authenticate_inner(message, policy)?.await {
            inner
        } else {
            Err(Error::Unknown)
        }
    }

    pub(crate) fn blocking_authenticate(&self, message: &str, policy: &Policy) -> Result<()> {
        // TODO: `result_flattening` feature
        if let Ok(inner) = self.authenticate_inner(message, policy)?.blocking_recv() {
            inner
        } else {
            Err(Error::Unknown)
        }
    }

    fn authenticate_inner(&self, message: &str, policy: &Policy) -> Result<Receiver> {
        log::error!("0");
        robius_android_env::with_activity(|env, context| {
            let (tx, rx) = callback::channel();

            log::error!("a: {env:#?} {context:#?}");
            let callback_class = callback::get_callback_class(env)?;

            log::error!("b");
            let callback_instance =
                construct_callback(env, callback_class, Box::into_raw(Box::new(tx)))?;
            log::error!("c");
            let cancellation_signal = construct_cancellation_signal(env)?;
            log::error!("d");
            let executor = get_executor(env, context)?;
            log::error!("e");

            let biometric_prompt = construct_biometric_prompt(env, context, policy, message)?;
            log::error!("f");

            let temp = env.call_method(
                biometric_prompt,
                "authenticate",
                "(Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;Landroid/hardware/\
                 biometrics/BiometricPrompt$AuthenticationCallback;)V",
                &[
                    JValueGen::Object(&cancellation_signal),
                    JValueGen::Object(&executor),
                    JValueGen::Object(&callback_instance),
                ],
            )?;
            log::error!("g: {temp:#?}");

            Ok(rx)
        })
        .ok_or(Error::Unknown)?
    }
}

#[derive(Debug)]
pub(crate) struct Policy {
    #[allow(dead_code)]
    strength: BiometricStrength,
    password: bool,
}

#[derive(Debug)]
pub(crate) struct PolicyBuilder {
    biometrics: Option<BiometricStrength>,
    password: bool,
}

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self {
            biometrics: Some(BiometricStrength::Strong),
            password: true,
        }
    }

    pub(crate) const fn biometrics(self, biometrics: Option<BiometricStrength>) -> Self {
        Self { biometrics, ..self }
    }

    pub(crate) const fn password(self, password: bool) -> Self {
        Self { password, ..self }
    }

    pub(crate) const fn watch(self, _: bool) -> Self {
        self
    }

    pub(crate) const fn wrist_detection(self, _: bool) -> Self {
        self
    }

    pub(crate) const fn build(self) -> Option<Policy> {
        // if self.password {
        if let Some(strength) = self.biometrics {
            return Some(Policy {
                strength,
                password: self.password,
            });
        }
        None
        // }
        // None
    }
}

fn construct_callback<'a>(
    env: &mut JNIEnv<'a>,
    class: &GlobalRef,
    channel_ptr: *mut Sender,
) -> Result<JObject<'a>> {
    env.new_object(class, "(J)V", &[JValueGen::Long(channel_ptr as i64)])
        .map_err(|e| e.into())
}

fn construct_cancellation_signal<'a>(env: &mut JNIEnv<'a>) -> Result<JObject<'a>> {
    env.new_object("android/os/CancellationSignal", "()V", &[])
        .map_err(|e| e.into())
}

fn get_executor<'a, 'o, O>(env: &mut JNIEnv<'a>, context: O) -> Result<JObject<'a>>
where
    O: AsRef<JObject<'o>>,
{
    env.call_method(
        context,
        "getMainExecutor",
        "()Ljava/util/concurrent/Executor;",
        &[],
    )?
    .l()
    .map_err(|e| e.into())
}

fn construct_biometric_prompt<'a>(
    env: &mut JNIEnv<'a>,
    context: &JObject<'_>,
    policy: &Policy,
    message: &str,
) -> Result<JObject<'a>> {
    let context = env.new_global_ref(context).unwrap();
    log::error!("what");
    let builder = env.new_object(
        "android/hardware/biometrics/BiometricPrompt$Builder",
        "(Landroid/content/Context;)V",
        &[JValueGen::Object(context.as_ref())],
    )?;

    // TODO: Custom title and subtitle
    let title = env.new_string("temp")?;

    env.call_method(
        &builder,
        "setTitle",
        "(Ljava/lang/CharSequence;)Landroid/hardware/biometrics/BiometricPrompt$Builder;",
        &[JValueGen::Object(&title)],
    )?;

    const STRONG: i32 = 0xf;
    const WEAK: i32 = 0xff;
    // TODO: We require password authentication for now otherwise we would also have
    // to pass a cancel callback.
    const CREDENTIAL: i32 = 0x8000;

    // env.call_method(
    //     &builder,
    //     "setAllowedAuthenticators",
    //     "(I)Landroid/hardware/biometrics/BiometricPrompt$Builder;",
    //     &[JValueGen::Int(
    //         match policy.strength {
    //             BiometricStrength::Strong => STRONG,
    //             BiometricStrength::Weak => WEAK,
    //         } | if policy.password { CREDENTIAL } else { 0 }
    //             | 0x80ff,
    //     )],
    // )?;
    env.call_method(
        &builder,
        "setAllowedAuthenticators",
        "(I)Landroid/hardware/biometrics/BiometricPrompt$Builder;",
        // TODO: We require password authentication for now otherwise we would also have to pass a
        // cancel callback.
        &[JValueGen::Int(0x0000000f | 0x000000ff | 0x00008000)],
    )?;

    env.call_method(
        builder,
        "build",
        "()Landroid/hardware/biometrics/BiometricPrompt;",
        &[],
    )?
    .l()
    .map_err(|e| e.into())
}
