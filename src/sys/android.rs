mod callback;
// TODO: Ideally we remove this whole module once we can test in Makepad.
mod test;

use crate::{BiometricStrength, Error, Result};
use callback::{Receiver, Sender};
use jni::{objects::JValueGen, JNIEnv};

pub use jni::{JavaVM, objects::{GlobalRef, JObject}};

// Note: we could add a <'j> lifetime param to this,
// but then we'd have to add it to the top-level RawContext type too.
pub(crate) type RawContext = (JavaVM, ActivityObject<'static>);

pub enum ActivityObject<'j> {
    JObject(JObject<'j>),
    GlobalRef(GlobalRef),
}

pub(crate) struct Context {
    vm: JavaVM,
    context: GlobalRef,
}

impl Context {
    pub(crate) fn new(inner: RawContext) -> Self {
        let (vm, activity_obj) = inner;
        let context = match activity_obj {
            ActivityObject::JObject(jobject) => {
                let env = vm.get_env().unwrap();
                env.new_global_ref(jobject).unwrap()
            }
            ActivityObject::GlobalRef(g) => g,
        };
        Self { vm, context }
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
        // TODO: If we actually call blocking_recv, it blocks the main thread somehow
        // preventing the callback from being invoked and leading to deadlock.
        self.authenticate_inner(message, policy)?;
        // .blocking_recv()
        // .unwrap();
        Ok(())
    }

    fn authenticate_inner(&self, _message: &str, _policy: &Policy) -> Result<Receiver> {
        let mut env = self.vm.get_env()?;

        let (tx, rx) = callback::channel();

        let callback_class = callback::get_callback_class(&mut env)?;

        let callback_instance =
            construct_callback(&mut env, callback_class, Box::into_raw(Box::new(tx)))?;
        let cancellation_signal = construct_cancellation_signal(&mut env)?;
        let executor = get_executor(&mut env, &self.context)?;

        let biometric_prompt = construct_biometric_prompt(&mut env, &self.context)?;

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
}

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

fn get_executor<'a>(env: &mut JNIEnv<'a>, context: &GlobalRef) -> Result<JObject<'a>> {
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
    context: &GlobalRef,
) -> Result<JObject<'a>> {
    let builder = env.new_object(
        "android/hardware/biometrics/BiometricPrompt$Builder",
        "(Landroid/content/Context;)V",
        &[JValueGen::Object(context)],
    )?;

    // TODO: Custom title and subtitle
    let title = env.new_string("Rust authentication prompt")?;

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
