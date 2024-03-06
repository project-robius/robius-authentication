use std::sync::OnceLock;

use jni::{
    objects::{JClass, JObject, JValueGen},
    JNIEnv, JavaVM, NativeMethod,
};

use crate::{BiometricStrength, Result};

static VM: OnceLock<JavaVM> = OnceLock::new();

const AUTHENTICATION_CALLBACK_BYTECODE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));

#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _: std::ffi::c_void,
) -> jni::sys::jint {
    VM.set(unsafe { JavaVM::from_raw(vm) }.unwrap()).unwrap();
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

pub(crate) fn blocking_authenticate(
    context: JObject,
    _message: &str,
    _policy: &Policy,
) -> Result<()> {
    let vm = VM.get().unwrap();
    let mut env = vm.get_env().unwrap();

    let biometric_prompt = construct_biometric_prompt(&mut env, &context);

    let class = load_callback_class(&mut env);

    let constructor = get_constructor(&mut env, &class);
    let callback_instance = construct(&mut env, constructor, allocate_channel());

    let cancellation_signal = construct_cancellation_signal(&mut env);
    let executor = get_executor(&mut env, context);

    log::error!("try");

    let temp = env.register_native_methods(
        class,
        &[NativeMethod {
            name: "rustCallback".into(),
            sig: "()V".into(),
            fn_ptr: crate::Java_robius_authentication_AuthenticationCallback_rustCallback as *mut _,
        }],
    );

    log::error!("here");

    let res = env.call_method(
        biometric_prompt,
        "authenticate",
        "(Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;Landroid/hardware/\
         biometrics/BiometricPrompt$AuthenticationCallback;)V",
        &[
            JValueGen::Object(&cancellation_signal),
            JValueGen::Object(&executor),
            JValueGen::Object(&callback_instance),
        ],
    );
    log::info!("so?jlkla: {res:#?}");

    // Unable to start activity:
    //
    // java.lang.SecurityException: Must have USE_BIOMETRIC permission: Neither user
    // 10191 nor current process has android.permission.USE_BIOMETRIC.

    // https://android.googlesource.com/platform/frameworks/support/+/63add6e2590077c18556dcdd96aa5c6ff68eb13b/biometric/biometric/src/main/AndroidManifest.xml

    Ok(())
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

    log::error!("a");

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

fn get_constructor<'a>(env: &mut JNIEnv<'a>, callback_class: &JClass<'a>) -> JObject<'a> {
    let constructors = env
        .call_method(
            callback_class,
            "getConstructors",
            "()[Ljava/lang/reflect/Constructor;",
            &[],
        )
        .unwrap();

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

fn construct_biometric_prompt<'a>(env: &mut JNIEnv<'a>, context: &JObject<'a>) -> JObject<'a> {
    const BUILDER_CLASS: &str = "android/hardware/biometrics/BiometricPrompt$Builder";

    let builder = env
        .new_object(
            BUILDER_CLASS,
            "(Landroid/content/Context;)V",
            &[JValueGen::Object(&context)],
        )
        .unwrap();

    let title = env.new_string("HELLO FROM RUST").unwrap();

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

fn construct_cancellation_signal<'a>(env: &mut JNIEnv<'a>) -> JObject<'a> {
    env.new_object("android/os/CancellationSignal", "()V", &[])
        .unwrap()
}

fn get_executor<'a>(env: &mut JNIEnv<'a>, context: JObject<'a>) -> JObject<'a> {
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
