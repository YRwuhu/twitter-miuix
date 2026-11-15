//! Windows DPAPI 凭据持久化。
//!
//! 用 `CryptProtectMemory` / `CryptProtectData` 把 cookie 绑定到当前用户账户，
//! 密文存到 `%APPDATA%/twitter-miuix/session.bin`。
//! 其他 Windows 用户即使拿到文件也解不开。
#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::path::PathBuf;

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            data_in: *const DataBlob,
            description: *const u16,
            optional_entropy: *const DataBlob,
            reserved: *const c_void,
            prompt_struct: *const c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
        fn CryptUnprotectData(
            data_in: *const DataBlob,
            description: *mut u16,
            optional_entropy: *const DataBlob,
            reserved: *const c_void,
            prompt_struct: *const c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(handle: *const c_void) -> *const c_void;
    }

    #[repr(C)]
    struct DataBlob {
        size: u32,
        data: *mut u8,
    }

    fn session_path() -> Option<PathBuf> {
        let appdata = std::env::var_os("APPDATA")?;
        let dir = PathBuf::from(appdata).join("twitter-miuix");
        std::fs::create_dir_all(&dir).ok()?;
        Some(dir.join("session.bin"))
    }

    pub fn save(plaintext: &str) -> Result<(), String> {
        let path = session_path().ok_or("无法定位 APPDATA")?;
        let input = DataBlob {
            size: plaintext.len() as u32,
            data: plaintext.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            size: 0,
            data: std::ptr::null_mut(),
        };
        let desc: Vec<u16> = "Chuckle X session\0".encode_utf16().collect();

        let ok = unsafe {
            CryptProtectData(
                &input,
                desc.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut output,
            )
        };
        if ok == 0 {
            return Err("CryptProtectData 失败".into());
        }
        let bytes = unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }.to_vec();
        unsafe { LocalFree(output.data as *const c_void) };
        std::fs::write(path, bytes).map_err(|e| e.to_string())
    }

    pub fn load() -> Result<Option<String>, String> {
        let path = match session_path() {
            Some(p) => p,
            None => return Ok(None),
        };
        let bytes = match std::fs::read(path) {
            Ok(b) if !b.is_empty() => b,
            Ok(_) => return Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };

        let input = DataBlob {
            size: bytes.len() as u32,
            data: bytes.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            size: 0,
            data: std::ptr::null_mut(),
        };
        let ok = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut output,
            )
        };
        if ok == 0 {
            // 文件损坏或换了用户，直接当作没有登录态
            return Ok(None);
        }
        let plain =
            unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }.to_vec();
        unsafe { LocalFree(output.data as *const c_void) };
        Ok(String::from_utf8(plain).ok().filter(|s| !s.is_empty()))
    }

    pub fn clear() {
        if let Some(path) = session_path() {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(not(windows))]
mod win {
    pub fn save(_: &str) -> Result<(), String> {
        Err("非 Windows 平台不提供凭据持久化".into())
    }
    pub fn load() -> Result<Option<String>, String> {
        Ok(None)
    }
    pub fn clear() {}
}

pub use win::{clear, load, save};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn dpapi_roundtrip() {
        let original = "auth_token=test; ct0=abcdef";
        save(original).expect("保存失败");
        let loaded = load().expect("读取失败");
        assert_eq!(loaded.as_deref(), Some(original));
        clear();
        let after = load().expect("再次读取失败");
        assert_eq!(after, None);
    }
}
