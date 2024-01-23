// TODO: To prevent memory leaks, you must call WinBioFree to release the
// WINBIO_ASYNC_RESULT structure after you have finished using it.
//
// https://github.com/MicrosftDocs/win32/blob/docs/desktop-src/SecBioMet/creating-client-applications.md
//
// https://learn.microsoft.com/en-us/windows/win32/api/winbio/nf-winbio-winbiolocatesensor
// https://learn.microsoft.com/en-us/windows/win32/apiindex/windows-api-list#security-and-identity

use windows::{
    core::{HSTRING, PCSTR, PCWSTR, PSTR, PWSTR},
    Security::Credentials::UI::UserConsentVerifier,
    Win32::{
        Foundation::{HANDLE, HWND},
        Graphics::Gdi::HBITMAP,
        NetworkManagement::NetManagement::UNLEN,
        Security::{
            Authentication::Identity::{
                LsaConnectUntrusted, LsaLookupAuthenticationPackage, LSA_STRING,
                MSV1_0_PACKAGE_NAME,
            },
            Credentials::{
                CredUIPromptForCredentialsA, CredUIPromptForWindowsCredentialsA,
                CredUIPromptForWindowsCredentialsW, CREDUIWIN_FLAGS, CREDUI_FLAGS_ALWAYS_SHOW_UI,
                CREDUI_FLAGS_DO_NOT_PERSIST, CREDUI_FLAGS_GENERIC_CREDENTIALS,
                CREDUI_FLAGS_INCORRECT_PASSWORD, CREDUI_FLAGS_KEEP_USERNAME,
                CREDUI_FLAGS_VALIDATE_USERNAME, CREDUI_INFOW,
            },
        },
        System::WindowsProgramming::{GetUserNameA, GetUserNameW},
    },
};

use crate::BiometricStrength;

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

    // pub(crate) const fn wrist_detection(self, _: bool) -> Self {
    //  Self
    // }

    pub(crate) const fn build(self) -> Option<Policy> {
        Some(Policy)
    }
}

pub(crate) struct Policy;

pub(crate) struct Context;

impl Context {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn authenticate(&self, _: &Policy) -> bool {
        unimplemented!()
    }

    pub(crate) fn blocking_authenticate(&self, _: &Policy) -> bool {
        let handle = handle();
        let mut auth_package = auth_package(handle);
        let mut auth_package = 0u32;

        let mut out_auth_buffer = std::ptr::null_mut();
        let mut out_cred_size = 0u32;

        let mut message = [0u16; 14];

        let mut i = 0;
        for c in "message".encode_utf16() {
            message[i] = c;
            i += 1;
        }
        message[i] = 0;

        let mut caption = [0u16; 14];

        let mut i = 0;
        for c in "caption".encode_utf16() {
            caption[i] = c;
            i += 1;
        }
        caption[i] = 0;

        let ui = CREDUI_INFOW {
            cbSize: core::mem::size_of::<CREDUI_INFOW>() as u32,
            hwndParent: HWND(0),
            pszMessageText: PCWSTR(message.as_ptr()),
            pszCaptionText: PCWSTR(caption.as_ptr()),
            hbmBanner: HBITMAP(0),
        };

        println!("1");
        UserConsentVerifier::RequestVerificationAsync(&HSTRING::from_wide(&caption).unwrap())
            .unwrap();
        println!("ok");
        loop {}

        let mut user_name = user_name();

        let _y = unsafe {
            CredUIPromptForWindowsCredentialsW(
                Some(&ui as *const _),
                0,
                &mut auth_package as *mut _,
                None,
                0,
                &mut out_auth_buffer as *mut _,
                &mut out_cred_size as *mut _,
                None,
                CREDUIWIN_FLAGS(0x1000),
            )
        };

        println!("y: {_y:?}");
        println!("{out_auth_buffer:0x?}");
        println!("{out_cred_size}");

        unimplemented!()
    }
}

fn handle() -> HANDLE {
    let mut handle = HANDLE(0);

    let result = unsafe { LsaConnectUntrusted(&mut handle as *mut _) };

    if result.is_err() {
        panic!("{result:?}");
    } else {
        handle
    }
}

fn auth_package(handle: HANDLE) -> u32 {
    let mut auth_package = 0u32;

    let auth_package_bytes = unsafe { MSV1_0_PACKAGE_NAME.as_bytes() };
    let auth_package_len = auth_package_bytes.len();
    let auth_package_max_len = auth_package_len + 1;

    let mut auth_package_name = vec![0; auth_package_max_len];
    auth_package_name[..auth_package_len].copy_from_slice(auth_package_bytes);
    auth_package_name[auth_package_len] = 0;

    let str = LSA_STRING {
        Length: auth_package_len as u16,
        MaximumLength: auth_package_max_len as u16,
        Buffer: PSTR(auth_package_name.as_ptr() as *mut _),
    };

    let result = unsafe {
        LsaLookupAuthenticationPackage(handle, &str as *const _, &mut auth_package as *mut _)
    };

    if result.is_err() {
        panic!("{result:?}");
    } else {
        auth_package
    }
}

fn user_name() -> Vec<u16> {
    todo!();
    // let mut len = UNLEN + 1;
    // let mut user_name = Vec::with_capacity(len as usize);
    //
    // unsafe { GetUserNameW(PWSTR(user_name.as_ptr_mut()), &mut len as *mut _)?
    // };
    //
    // unsafe { user_name.set_len(len as usize) };
    // TODO: Do we bother doing this?
    // user_name.shrink_to_fit();
    // user_name
}

fn pack_authentication_buffer(user_name: &Vec<u16>) -> &'static [u8] {
    todo!();
}
