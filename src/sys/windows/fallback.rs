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
                CredPackAuthenticationBufferW, CredUIPromptForWindowsCredentialsW,
                CredUnPackAuthenticationBufferW, CREDUIWIN_FLAGS, CREDUI_INFOW,
                CRED_PACK_PROTECTED_CREDENTIALS,
            },
            LogonUserW, LOGON32_LOGON_BATCH, LOGON32_PROVIDER_DEFAULT,
        },
        System::WindowsProgramming::GetUserNameW,
    },
};
use windows_core::PWSTR;

use crate::{text::WindowsText, Result};

pub(super) fn authenticate(text: WindowsText) -> Result<()> {
    // _message and _caption can only be dropped after we've used ui.
    let (_message, _caption, ui) = ui(text);

    let handle = handle();
    let mut auth_package = auth_package(handle);

    let mut out_auth_buffer = std::ptr::null_mut();
    let mut out_cred_size = 0u32;

    let _error = unsafe {
        CredUIPromptForWindowsCredentialsW(
            Some(&ui as *const _),
            0,
            &mut auth_package as *mut _,
            // Some(&mut auth_buffer as *mut _ as *mut _),
            // 0,
            None,
            0,
            &mut out_auth_buffer as *mut _,
            &mut out_cred_size as *mut _,
            None,
            // CREDUIWIN_FLAGS(0x1000),
            CREDUIWIN_FLAGS(0x0 | 0x200),
        )
    };

    // TODO: Check _error

    let mut user_name = [0u16; 100];
    let mut user_name_size = user_name.len() as u32;

    let mut domain_name = [0u16; 100];
    let mut domain_name_size = domain_name.len() as u32;

    let mut password = [0u16; 100];
    let mut password_size = password.len() as u32;

    unsafe {
        CredUnPackAuthenticationBufferW(
            CRED_PACK_PROTECTED_CREDENTIALS,
            out_auth_buffer,
            out_cred_size,
            PWSTR(user_name.as_mut_ptr()),
            &mut user_name_size as *mut _,
            PWSTR(domain_name.as_mut_ptr()),
            Some(&mut domain_name_size as *mut _),
            PWSTR(password.as_mut_ptr()),
            &mut password_size as *mut _,
        )
    }
    .unwrap();

    println!(
        "username in credential buffer: {:#?}",
        String::from_utf16(&user_name[..user_name_size as usize])
    );
    println!(
        "provided password: {:#?}",
        String::from_utf16(&password[..password_size as usize])
    );
    println!(
        "domain name: {:#?}",
        String::from_utf16(&domain_name[..domain_name_size as usize])
    );

    let mut handle = HANDLE(0xdead);

    let (mut user_name, _user_name_size) = guser_name();
    if unsafe {
        LogonUserW(
            PWSTR(user_name.as_mut_ptr()),
            None,
            PWSTR(password.as_mut_ptr()),
            LOGON32_LOGON_BATCH,
            LOGON32_PROVIDER_DEFAULT,
            &mut handle as *mut _,
        )
    }
    .is_ok()
    {
        todo!("happy");
    } else {
        todo!("unhappy");
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

    if result.is_ok() {
        auth_package
    } else {
        todo!();
    }
}

fn ui(text: WindowsText) -> (Vec<u16>, Vec<u16>, CREDUI_INFOW) {
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

    (message, caption, ui)
}

fn guser_name() -> ([u16; 257], usize) {
    let mut user_name = [0u16; UNLEN as usize + 1];
    let mut user_name_size = user_name.len() as u32;
    // This expect is fine as the buffer is guaranteed to be large
    // enough: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getusernamew
    unsafe {
        GetUserNameW(
            PWSTR(&mut user_name as *mut _),
            &mut user_name_size as *mut _,
        )
    }
    .expect("user name buffer too small");

    (user_name, user_name_size as usize)
}

fn pack_authentication_buffer(mut user_name: [u16; 257]) -> ([u8; 1000], u32) {
    // TODO: Len
    let mut buf = [0u8; 1000];
    let mut buf_size = buf.len() as u32;

    let mut password = [0u16; 1];

    unsafe {
        CredPackAuthenticationBufferW(
            CRED_PACK_PROTECTED_CREDENTIALS,
            PWSTR(&mut user_name as *mut _),
            PWSTR(&mut password as *mut _),
            Some(&mut buf as *mut _),
            &mut buf_size,
        )
        .unwrap()
    };

    (buf, buf_size)
}
