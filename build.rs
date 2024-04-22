use std::{env, path::PathBuf};

const JAVA_FILE_RELATIVE_PATH: &str = "src/sys/android/AuthenticationCallback.java";

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "android" {
        println!("cargo:rerun-if-changed={JAVA_FILE_RELATIVE_PATH}");

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let java_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join(JAVA_FILE_RELATIVE_PATH);

        let android_jar_path = android_build::android_jar(None)
            .expect("Failed to find android.jar");
        
        // Compile the .java file into a .class file.
        let mut java_build = android_build::JavaBuild::new();
        java_build
            .class_path(android_jar_path.clone())
            .classes_out_dir(out_dir.clone())
            .file(java_file);
        // eprintln!("java_build: {:?}", java_build.command());
        java_build.compile().expect("javac invocation failed");

        let class_file = out_dir
            .join("robius")
            .join("authentication")
            .join("AuthenticationCallback.class");

        let d8_jar_path = android_build::android_d8_jar(None)
            .expect("Failed to find d8.jar");

        let mut java_run = android_build::JavaRun::new();
        java_run
            .class_path(d8_jar_path)
            .main_class("com.android.tools.r8.D8")
            .arg("--classpath")
            .arg(android_jar_path)
            .arg("--output")
            .arg(&out_dir)
            .arg(&class_file);
        // eprintln!("java_run: {:?}", java_run.command());
        java_run.run().expect("java d8.jar invocation failed");
    }
}
