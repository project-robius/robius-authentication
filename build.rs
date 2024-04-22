use std::{env, path::PathBuf, process::Command};

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
        java_build.class_paths.push(android_jar_path.clone());
        java_build.classes_out_dir = Some(out_dir.clone());
        java_build.files.push(java_file);
        eprintln!("java_build: {:?}", java_build.command());
        java_build.compile().expect("javac invocation failed");

        let class_file = out_dir
            .join("robius")
            .join("authentication")
            .join("AuthenticationCallback.class");

        let d8_jar_path = android_build::android_d8_jar(None)
            .expect("Failed to find d8.jar");

        let java_path = android_build::java()
            .expect("Failed to find the `java` executable");

        // TODO: once android-build suppors running a Java command, switch to that.
        // Compile the .class file into a .dex file.
        assert!(
            Command::new(java_path)
                .arg("-cp")
                .arg(d8_jar_path)
                .arg("com.android.tools.r8.D8")
                .arg("--classpath")
                .arg(android_jar_path)
                .arg("--output")
                .arg(&out_dir)
                .arg(&class_file)
                .output()
                .unwrap()
                .status
                .success(),
            "java d8.jar invocation failed"
        );
    }
}
