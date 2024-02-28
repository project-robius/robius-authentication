There are three main parts to getting a working application on iOS: bundles, code signing, and provisioning profiles.

### Bundles

Apple Documentation: <https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/Introduction/Introduction.html#//apple_ref/doc/uid/10000123i>

Bundles have the following format:
```
Bundle
  ├── Executable
  ├── Info.plist
  └── Other resource files
```

They shouldn't be hard to generate e.g. `cargo-bundle:
https://github.com/burtonageo/cargo-bundle/blob/master/src/bundle/ios_bundle.rs


### Code Signing

Pretty simple to do using the `codesign` utility.

### Provisioning Profile

This is the hard part. In order to run on a real device, the app must have an associated provisioning profile. Provisioning profiles are managed in the cloud by Apple, but they do have an API (https://developer.apple.com/documentation/appstoreconnectapi/profiles).

Some documentation:
- https://devcenter.bitrise.io/en/code-signing/ios-code-signing/generating-ios-code-signing-files.html
- https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles:
    > You create provisioning profiles using the Apple Developer website, either directly using the website or indirectly using Xcode or the App Store Connect API.

AFAICT you only need code signing and a provisioning profile to test on a real iPhone - the simulator is happy to run an unsigned bundle.

code signing vs provisioning profile: https://codesigningstore.com/difference-between-code-signing-identities-and-provisioning-profiles
