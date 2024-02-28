use std::sync::mpsc::channel;

use jni::{
    objects::{JObject, JString, JValueGen},
    strings::JNIString,
    JNIEnv, JavaVM,
};

use crate::{BiometricStrength, Result};

static mut VM: *mut jni::sys::JavaVM = std::ptr::null_mut();

const AUTHENTICATION_CALLBACK_BYTECODE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));

#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _: std::ffi::c_void,
) -> jni_sys::jint {
    VM = vm as *mut _ as _;
    // TODO
    jni::sys::JNI_VERSION_1_6 as _
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

pub(crate) async fn authenticate(_message: &str, _policy: &Policy) -> Result<()> {
    unimplemented!()
}

pub(crate) fn blocking_authenticate(_message: &str, _policy: &Policy) -> Result<()> {
    let vm_ptr = unsafe { VM };
    assert!(!vm_ptr.is_null());
    let vm = unsafe { JavaVM::from_raw(vm_ptr) }.unwrap();
    let mut env = vm.get_env().unwrap();

    const BIOMETRIC_PROMPT_CLASS: &str = "android/hardware/biometrics/BiometricPrompt";
    const AUTHENTICATION_FUNCTION: &str = "authenticate";

    let class = load_callback_class(&mut env);
    let constructor = get_constructor(&mut env, class);
    let callback_instance = construct(&mut env, constructor, allocate_channel());

    let authenticate_signature = "(Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;\
                                  Landroid/hardware/biometrics/\
                                  BiometricPrompt$AuthenticationCallback;)V";

    Ok(())
}

fn load_callback_class<'a>(env: &mut JNIEnv<'a>) -> JObject<'a> {
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
            env.new_string("AuthenticationCallback").unwrap(),
        ))],
    )
    .unwrap()
    .l()
    .unwrap()
}

fn get_constructor<'a>(env: &mut JNIEnv<'a>, callback_class: JObject<'a>) -> JObject<'a> {
    let constructors = env
        .call_method(
            callback_class,
            "getConstructors",
            "()[Ljava/lang/reflect/Constructor;",
            &[],
        )
        .unwrap();

    log::error!("{constructors:?}");

    env.call_static_method(
        "java/lang/reflect/Array",
        "get",
        "(Ljava/lang/Object;I)Ljava/lang/Object;",
        &[constructors.borrow(), JValueGen::Int(0)],
    )
    .unwrap()
    .l()
    .unwrap()
}

fn allocate_channel() -> i64 {
    0xdeadbeef
}

fn construct<'a>(env: &mut JNIEnv<'a>, constructor: JObject<'a>, channel_ptr: i64) -> JObject<'a> {
    let default = env
        .call_static_method(
            "java/lang/Long",
            "valueOf",
            "(J)Ljava/lang/Long;",
            &[JValueGen::Long(channel_ptr)],
        )
        .unwrap()
        .l()
        .unwrap();

    let constructor_parameters = JValueGen::Object(JObject::from(
        env.new_object_array(1, "java/lang/Long", default).unwrap(),
    ));

    env.call_method(
        constructor,
        "newInstance",
        "([Ljava/lang/Object;)Ljava/lang/Object;",
        &[constructor_parameters.borrow()],
    )
    .unwrap()
    .l()
    .unwrap()
}
