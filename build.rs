use std::{env, process::Command};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "android" {
        println!("cargo:rerun-if-changed=AuthenticationCallback.java");

        let out_dir = env::var("OUT_DIR").unwrap();
        let java_file = format!(
            "{}/AuthenticationCallback.java",
            env::var("CARGO_MANIFEST_DIR").unwrap()
        );

        // TODO: Find programatically.
        let android_sdk_path = "/Users/klim/Library/Android/sdk";
        // TODO: Version
        let d8_path = format!("{android_sdk_path}/build-tools/34.0.0/d8");
        // TODO: Version
        let android_jar_path = format!("{android_sdk_path}/platforms/android-34/android.jar");

        assert!(
            Command::new("javac")
                .args(["-cp", &android_jar_path, &java_file, "-d", &out_dir,])
                .output()
                .unwrap()
                .status
                .success(),
            "javac invocation failed"
        );

        let class_file = format!("{out_dir}/AuthenticationCallback.class");

        assert!(
            Command::new(d8_path)
                .args(["--output", &out_dir, &class_file])
                .output()
                .unwrap()
                .status
                .success(),
            "d8 invocation failed"
        );

        println!("cargo:warning={out_dir}");
    }
}
