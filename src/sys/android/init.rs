use std::sync::OnceLock;

use jni::{
    objects::{GlobalRef, JClass, JObject, JValueGen},
    sys::{jboolean, jint, jlong},
    JNIEnv, JavaVM, NativeMethod,
};
use tokio::sync::oneshot::Sender;

use crate::Result;

const AUTHENTICATION_CALLBACK_BYTECODE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));

static VM: OnceLock<State> = OnceLock::new();

#[derive(Debug)]
pub(super) struct State {
    pub(super) vm: JavaVM,
    pub(super) callback_class: GlobalRef,
}

#[no_mangle]
unsafe extern "C" fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, _: std::ffi::c_void) -> jint {
    if on_load(vm).is_ok() {
        jni::sys::JNI_VERSION_1_6 as _
    } else {
        -1
    }
}

fn on_load(vm: *mut jni::sys::JavaVM) -> Result<()> {
    let vm = unsafe { JavaVM::from_raw(vm) }?;
    let mut env = vm.get_env()?;

    let callback_class = load_callback_class(&mut env)?;
    register_rust_callback(&mut env, &callback_class)?;

    let global = env.new_global_ref(callback_class)?;

    VM.set(State {
        vm,
        callback_class: global,
    })
    // TODO
    .unwrap();

    Ok(())
}

pub(super) fn get_vm() -> &'static State {
    VM.get().unwrap()
}

// NOTE: This must be kept in sync with the signature of `rust_callback`.
const RUST_CALLBACK_SIGNATURE: &str = "(JIZI)V";

// NOTE: The signature of this function must be kept in sync with
// `RUST_CALLBACK_SIGNATURE`.
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

fn register_rust_callback<'a>(env: &mut JNIEnv<'a>, callback_class: &JClass<'a>) -> Result<()> {
    env.register_native_methods(
        callback_class,
        &[NativeMethod {
            name: "rustCallback".into(),
            sig: RUST_CALLBACK_SIGNATURE.into(),
            fn_ptr: rust_callback as *mut _,
        }],
    )
    .map_err(|e| e.into())
}

fn load_callback_class<'a>(env: &mut JNIEnv<'a>) -> Result<JClass<'a>> {
    const LOADER_CLASS: &str = "dalvik/system/InMemoryDexClassLoader";

    let byte_buffer = unsafe {
        env.new_direct_byte_buffer(
            AUTHENTICATION_CALLBACK_BYTECODE.as_ptr() as *mut u8,
            AUTHENTICATION_CALLBACK_BYTECODE.len(),
        )
    }?;

    let dex_class_loader = env.new_object(
        LOADER_CLASS,
        "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
        &[
            JValueGen::Object(&JObject::from(byte_buffer)),
            JValueGen::Object(&JObject::null()),
        ],
    )?;

    Ok(env
        .call_method(
            &dex_class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValueGen::Object(&JObject::from(
                env.new_string("robius/authentication/AuthenticationCallback")
                    .unwrap(),
            ))],
        )?
        .l()?
        .into())
}
