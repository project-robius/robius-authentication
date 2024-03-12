use jni::{
    objects::{JClass, JObject},
    JNIEnv,
};

use crate::{ActivityObject, Context, PolicyBuilder};

#[no_mangle]
pub unsafe extern "C" fn Java_com_example_myapplication2_Test_greeting<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    context: JObject<'static>,
) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Error),
    );

    let policy = PolicyBuilder::new().build().unwrap();
    let input = ActivityObject::GlobalRef(env.new_global_ref(context).unwrap());

    Context::new((env.get_java_vm().expect("couldn't get Java VM"), input))
        .blocking_authenticate("rust authentication message", &policy)
        .unwrap();
}
