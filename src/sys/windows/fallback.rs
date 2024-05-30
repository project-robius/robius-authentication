use windows::{
    core::{PCWSTR, PSTR},
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
                CredPackAuthenticationBufferW, CredUIPromptForWindowsCredentialsW, CREDUIWIN_FLAGS,
                CREDUI_INFOW, CRED_PACK_PROTECTED_CREDENTIALS,
            },
        },
        System::WindowsProgramming::GetUserNameW,
    },
};
use windows_core::PWSTR;

use crate::{text::WindowsText, Result};

pub(super) fn authenticate(text: WindowsText) -> Result<()> {
    let handle = handle();
    let mut auth_package = auth_package(handle);
    println!("auth_package: {auth_package}");
    // let mut auth_package = 0u32;

    let mut out_auth_buffer = std::ptr::null_mut();
    let mut out_cred_size = 0u32;

    let mut message = Vec::with_capacity(text.description.len() + 1);
    message.extend(text.description.encode_utf16());
    message.push(0);

    let mut caption = Vec::with_capacity(text.title.len() + 1);
    caption.extend(text.title.encode_utf16());
    caption.push(0);

    let ui = CREDUI_INFOW {
        cbSize: core::mem::size_of::<CREDUI_INFOW>() as u32,
        hwndParent: HWND(0),
        pszMessageText: PCWSTR(message.as_ptr()),
        pszCaptionText: PCWSTR(caption.as_ptr()),
        hbmBanner: HBITMAP(0),
    };

    // let mut user_name = user_name();
    // let mut r = &mut out_auth_buffer as *mut i32 as *mut c_void;

    let mut user_name = [0u16; UNLEN as usize + 1];
    let mut user_name_len = user_name.len() as u32;
    // This expect is fine as the buffer is guaranteed to be large
    // enough: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getusernamew
    unsafe {
        GetUserNameW(
            PWSTR(&mut user_name as *mut _),
            &mut user_name_len as *mut _,
        )
    }
    .expect("user name buffer too small");

    println!("hello");

    // TODO: Len
    let mut credential_in = [0u8; 1000];
    let mut credential_in_size = credential_in.len() as u32;

    let mut password = [0u16; 1];

    unsafe {
        CredPackAuthenticationBufferW(
            CRED_PACK_PROTECTED_CREDENTIALS,
            PWSTR(&mut user_name as *mut _),
            PWSTR(&mut password as *mut _),
            Some(&mut credential_in as *mut _),
            &mut credential_in_size,
        )
        .unwrap()
    };

    let _y = unsafe {
        CredUIPromptForWindowsCredentialsW(
            Some(&ui as *const _),
            0,
            &mut auth_package as *mut _,
            Some(&mut credential_in as *mut _ as *mut _),
            0,
            &mut out_auth_buffer as *mut _,
            &mut out_cred_size as *mut _,
            None,
            // CREDUIWIN_FLAGS(0x1000),
            CREDUIWIN_FLAGS(0x0 | 0x200),
        )
    };

    println!("y: {_y:?}");
    println!("{out_auth_buffer:0x?}");
    println!("{out_cred_size}");

    unimplemented!()
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

    if result.is_ok() {
        auth_package
    } else {
        todo!();
    }
}

fn user_name() -> Vec<u16> {
    todo!();
}

fn pack_authentication_buffer(user_name: &Vec<u16>) -> &'static [u8] {
    todo!();
}
