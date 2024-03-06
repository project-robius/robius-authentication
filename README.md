1. Add `USE_BIOMETRIC` permission to `AndroidManifest.xml` (in your application) e.g.:
    ```xml
    <?xml version="1.0" encoding="utf-8"?>
    <manifest xmlns:android="http://schemas.android.com/apk/res/android"
        xmlns:tools="http://schemas.android.com/tools">

        <uses-permission android:name="android.permission.USE_BIOMETRIC" />

        <application>
            <!-- ... -->
        </application>

    </manifest>
    ```
2. Change `android_sdk_path` in `build.rs` to your local SDK path (specifically version 34).
3. Generate `NDK/` subdirectory i.e. in `robius-authentication` project directory run
   ```bash
   /path/to/sdk/ndk/a-bunch-of-numbers/build/tools/make_standalone_toolchain.py --api 34 --arch arm64 --install-dir NDK/arm64
   ```
4. Change `ar` and `linker` paths in `.cargo/config.toml` to point to generated tools in `NDK/` subdirectory.

`cargo build` should now correctly generate a dynamic library that can be loaded into Android.