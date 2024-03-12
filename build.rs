use std::{env, process::Command};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    for (key, value) in env::vars() {
        eprintln!("{}: {}", key, value);
    }

    if target_os == "android" {
        println!("cargo:rerun-if-changed=AuthenticationCallback.java");

        let out_dir = env::var("OUT_DIR").unwrap();
        let java_file = format!(
            "{}/AuthenticationCallback.java",
            env::var("CARGO_MANIFEST_DIR").unwrap()
        );

        let android_sdk_home_env = env::var("ANDROID_HOME")
            .or_else(|_| env::var("ANDROID_SDK_ROOT"))
            .expect("ANDROID_HOME or ANDROID_SDK_ROOT must be set");
        let android_sdk_version_env = env::var("ANDROID_SDK_VERSION");
        let android_api_level_env   = env::var("ANDROID_API_LEVEL");

        let android_sdk_home        = android_sdk_home_env.as_str();
        let android_sdk_version     = android_sdk_version_env.as_deref();
        let android_api_level       = android_api_level_env.as_deref();

        // Try to find the d8 jar using an env var or from the SDK root directory.
        let d8_jar_path = env::var("ANDROID_D8_JAR").unwrap_or_else(|_|
            format!("{android_sdk_home}/build-tools/{}/lib/d8.jar",
                android_sdk_version.expect("ANDROID_SDK_VERSION must be set if ANDROID_D8_JAR is not set")
            )
        );

        // Try to find the android JAR using an env var or from the SDK root directory.
        let android_jar_path = env::var("ANDROID_JAR").unwrap_or_else(|_|
            format!("{android_sdk_home}/platforms/{}/android.jar",
                android_api_level.expect("ANDROID_API_LEVEL must be set if ANDROID_JAR is not set"),
            )
        );

        // Try to find `javac` in "JAVA_HOME/bin/", otherwise use the `javac` in the current path.
        let javac_path = env::var("JAVA_HOME")
            .map(|java_home| format!("{}/bin/javac", java_home))
            .unwrap_or_else(|_| "javac".to_string());

        // Try to find `java` in "JAVA_HOME/bin/", otherwise use the `java` in the current path.
        let java_path = env::var("JAVA_HOME")
            .map(|java_home| format!("{}/bin/java", java_home))
            .unwrap_or_else(|_| "java".to_string());

        eprintln!("javac_path: {}", javac_path);
        eprintln!("java_path: {}", java_path);

        // Compile the java_file into a class file.
        let mut javac_cmd = Command::new(&javac_path);
        javac_cmd
            .args(["-cp", &android_jar_path, &java_file, "-d", &out_dir,]);
        eprintln!("javac_cmd: {javac_cmd:?}");

        
        let javac_output = javac_cmd.output();

        eprintln!("javac_output: {javac_output:?}");

        assert!(   
            javac_output
                .unwrap()
                .status
                .success(),
            "javac invocation failed"
        );

        let class_file = format!("{out_dir}/robius/authentication/AuthenticationCallback.class");

        eprintln!("class_file: {}", class_file);
        eprintln!("d8_jar: {}", d8_jar_path);

        // Convert the class file into a dex file using d8.
        assert!(
            Command::new(java_path)
                .args(&[
                    "-cp",
                    &d8_jar_path,
                    "com.android.tools.r8.D8",
                    "--classpath",
                    &android_jar_path,
                    "--output",  
                    &out_dir,
                    &class_file,
                ])
                .output()
                .unwrap()
                .status
                .success(),
            "java d8.jar invocation failed"
        );


        // // Convert the class file into a dex file using d8.
        // let mut d8_cmd = Command::new(d8_path);
        // d8_cmd.args(["--output", &out_dir, &class_file]);
        // eprintln!("d8_cmd: {d8_cmd:?}");
        // let output_res = d8_cmd.output();
        // eprintln!("d8 output: {output_res:?}");
        // let output = output_res.unwrap();
        // eprintln!("\nd8 stdout:\n");
        // use std::io::Write;
        // std::io::stderr().write_all(&output.stdout).unwrap();
        // eprintln!("\nd8 stderr:\n");
        // std::io::stderr().write_all(&output.stderr).unwrap();
        // assert!(
        //     output
        //         .status
        //         .success(),
        //     "d8 invocation failed"
        // );
    }
}
