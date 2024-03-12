use std::sync::OnceLock;

use jni::{
    objects::{JClass, JObject, JValueGen},
    sys::{jboolean, jint, jlong},
    JNIEnv, JavaVM, NativeMethod,
};
use tokio::sync::oneshot::{Receiver, Sender};

use crate::{BiometricStrength, Result};

static VM: OnceLock<JavaVM> = OnceLock::new();

const AUTHENTICATION_CALLBACK_BYTECODE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));

#[no_mangle]
#[cfg(any(crate_type = "dylib", crate_type = "cdylib"))]
#[doc(hidden)]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, _: std::ffi::c_void) -> jint {
    VM.set(unsafe { JavaVM::from_raw(vm) }.unwrap()).unwrap();
    // TODO
    jni::sys::JNI_VERSION_1_6 as _
}

pub unsafe fn set_java_vm(vm: *mut u8) {
    VM.set(
        JavaVM::from_raw(vm.cast())
            .expect("Failed to create Java VM from raw pointer")
    )
    .expect("Failed to set global Java VM instance; was it already set?");
}

const RUST_CALLBACK_SIGNATURE: &str = "(JIZI)V";

unsafe extern "C" fn rust_callback<'a>(
    _: JNIEnv<'a>,
    _: JObject<'a>,
    channel_ptr: jlong,
    error_code: jint,
    failed: jboolean,
    help_code: jint,
) {
    log::error!(
        "rust callback invoked: {channel_ptr:#?} {error_code:#?} {failed:#?} {help_code:#?}"
    );

    let channel = unsafe { Box::from_raw(channel_ptr as *mut Sender<()>) };
    let _ = channel.send(());
}

pub(crate) struct Policy;

#[derive(Debug)]
pub(crate) struct PolicyBuilder;

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) const fn biometrics(self, _: Option<BiometricStrength>) -> Self {
        Self
    }

    pub(crate) const fn password(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn watch(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn wrist_detection(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn build(self) -> Option<Policy> {
        Some(Policy)
    }
}

pub(crate) async fn authenticate(
    context: JObject<'_>,
    message: &str,
    policy: &Policy,
) -> Result<()> {
    authenticate_inner(context, message, policy)?.await.unwrap();
    Ok(())
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
    let vm = VM.get().unwrap();
    let mut env = vm.get_env().unwrap();

    let callback_class = load_callback_class(&mut env);
    register_rust_callback(&mut env, &callback_class);

    let (tx, rx) = tokio::sync::oneshot::channel();

    let callback_instance =
        construct_callback(&mut env, &callback_class, Box::into_raw(Box::new(tx)));
    let cancellation_signal = construct_cancellation_signal(&mut env);
    let executor = get_executor(&mut env, &context);

    let biometric_prompt = construct_biometric_prompt(&mut env, &context);

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
    )
    .unwrap();
    Ok(rx)
}

fn load_callback_class<'a>(env: &mut JNIEnv<'a>) -> JClass<'a> {
    const LOADER_CLASS: &str = "dalvik/system/InMemoryDexClassLoader";

    let byte_buffer = unsafe {
        env.new_direct_byte_buffer(
            // TODO: Does AUTHENTICATION_CALLBACK_BYTECODE not need to be a static?
            AUTHENTICATION_CALLBACK_BYTECODE.as_ptr() as *mut u8,
            AUTHENTICATION_CALLBACK_BYTECODE.len(),
        )
    }
    .unwrap();

    let dex_class_loader = env
        .new_object(
            LOADER_CLASS,
            "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
            &[
                JValueGen::Object(&JObject::from(byte_buffer)),
                JValueGen::Object(&JObject::null()),
            ],
        )
        .unwrap();

    env.call_method(
        &dex_class_loader,
        "loadClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        &[JValueGen::Object(&JObject::from(
            env.new_string("robius/authentication/AuthenticationCallback")
                .unwrap(),
        ))],
    )
    .unwrap()
    .l()
    .unwrap()
    .into()
}

fn register_rust_callback<'a>(env: &mut JNIEnv<'a>, callback_class: &JClass<'a>) {
    env.register_native_methods(
        callback_class,
        &[NativeMethod {
            name: "rustCallback".into(),
            sig: RUST_CALLBACK_SIGNATURE.into(),
            fn_ptr: rust_callback as *mut _,
        }],
    )
    .unwrap();
}

fn construct_callback<'a>(
    env: &mut JNIEnv<'a>,
    class: &JClass<'a>,
    channel_ptr: *mut Sender<()>,
) -> JObject<'a> {
    env.new_object(class, "(J)V", &[JValueGen::Long(channel_ptr as i64)])
        .unwrap()
}

fn construct_cancellation_signal<'a>(env: &mut JNIEnv<'a>) -> JObject<'a> {
    env.new_object("android/os/CancellationSignal", "()V", &[])
        .unwrap()
}

fn get_executor<'a>(env: &mut JNIEnv<'a>, context: &JObject<'a>) -> JObject<'a> {
    env.call_method(
        context,
        "getMainExecutor",
        "()Ljava/util/concurrent/Executor;",
        &[],
    )
    .unwrap()
    .l()
    .unwrap()
}

fn construct_biometric_prompt<'a>(env: &mut JNIEnv<'a>, context: &JObject<'a>) -> JObject<'a> {
    let builder = env
        .new_object(
            "android/hardware/biometrics/BiometricPrompt$Builder",
            "(Landroid/content/Context;)V",
            &[JValueGen::Object(context)],
        )
        .unwrap();

    // TODO: Custom title and subtitle
    let title = env.new_string("Rust authentication prompt").unwrap();

    env.call_method(
        &builder,
        "setTitle",
        "(Ljava/lang/CharSequence;)Landroid/hardware/biometrics/BiometricPrompt$Builder;",
        &[JValueGen::Object(&title)],
    )
    .unwrap();

    env.call_method(
        &builder,
        "setAllowedAuthenticators",
        "(I)Landroid/hardware/biometrics/BiometricPrompt$Builder;",
        // TODO: We require password authentication for now otherwise we would also have to pass a
        // cancel callback.
        &[JValueGen::Int(0x0000000f | 0x000000ff | 0x00008000)],
    )
    .unwrap();

    env.call_method(
        builder,
        "build",
        "()Landroid/hardware/biometrics/BiometricPrompt;",
        &[],
    )
    .unwrap()
    .l()
    .unwrap()
}
