mod init;

use jni::{
    objects::{GlobalRef, JObject, JValueGen},
    JNIEnv,
};
use tokio::sync::oneshot::{Receiver, Sender};

use crate::{BiometricStrength, Error, Result};

pub type Context = GlobalRef;

#[derive(Debug)]
pub(crate) struct Policy {
    strength: BiometricStrength,
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
        if self.password {
            if let Some(strength) = self.biometrics {
                return Some(Policy { strength });
            }
        }
        None
    }
}

pub(crate) async fn authenticate(
    context: JObject<'_>,
    message: &str,
    policy: &Policy,
) -> Result<()> {
    authenticate_inner(context, message, policy)?
        .await
        // TODO: Custom error type.
        .map_err(|_| Error::Unknown)
}

pub(crate) fn blocking_authenticate(
    context: JObject<'_>,
    message: &str,
    policy: &Policy,
) -> Result<()> {
    // TODO: If we actually call blocking_recv, it blocks the main thread somehow
    // preventing the callback from being invoked and leading to deadlock.
    authenticate_inner(context, message, policy)?;
    // .blocking_recv()
    // .unwrap();
    Ok(())
}

fn authenticate_inner(context: JObject, _message: &str, _policy: &Policy) -> Result<Receiver<()>> {
    let init::State { vm, callback_class } = init::get_vm();
    let mut env = vm.get_env()?;

    let (tx, rx) = tokio::sync::oneshot::channel();

    let callback_instance =
        construct_callback(&mut env, callback_class, Box::into_raw(Box::new(tx)))?;
    let cancellation_signal = construct_cancellation_signal(&mut env)?;
    let executor = get_executor(&mut env, &context)?;

    let biometric_prompt = construct_biometric_prompt(&mut env, &context)?;

    env.call_method(
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
    Ok(rx)
}

fn construct_callback<'a>(
    env: &mut JNIEnv<'a>,
    class: &GlobalRef,
    channel_ptr: *mut Sender<()>,
) -> Result<JObject<'a>> {
    env.new_object(class, "(J)V", &[JValueGen::Long(channel_ptr as i64)])
        .map_err(|e| e.into())
}

fn construct_cancellation_signal<'a>(env: &mut JNIEnv<'a>) -> Result<JObject<'a>> {
    env.new_object("android/os/CancellationSignal", "()V", &[])
        .map_err(|e| e.into())
}

fn get_executor<'a>(env: &mut JNIEnv<'a>, context: &JObject<'a>) -> Result<JObject<'a>> {
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
    context: &JObject<'a>,
) -> Result<JObject<'a>> {
    let builder = env.new_object(
        "android/hardware/biometrics/BiometricPrompt$Builder",
        "(Landroid/content/Context;)V",
        &[JValueGen::Object(context)],
    )?;

    // TODO: Custom title and subtitle
    let title = env.new_string("Rust authentication prompt").unwrap();

    env.call_method(
        &builder,
        "setTitle",
        "(Ljava/lang/CharSequence;)Landroid/hardware/biometrics/BiometricPrompt$Builder;",
        &[JValueGen::Object(&title)],
    )?;

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
