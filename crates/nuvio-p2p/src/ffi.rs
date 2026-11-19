use crate::{P2pError, Result};
use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

/// FFI wrapper around the existing `libnuvio_engine.so` / `nuvio_engine.dll` C ABI.
///
/// Only symbols whose signatures are simple C primitives (`nuvio_engine_api_version`,
/// `nuvio_engine_protocol_backend_version`, `nuvio_engine_status_message`) are invoked directly
/// until the C header `nuvio_engine/nuvio_engine.h` struct layouts are added to the repository.
pub struct NativeNuvioEngineLibrary {
    lib: Library,
}

impl NativeNuvioEngineLibrary {
    /// Loads a native `nuvio_engine` shared library from `path`.
    ///
    /// # Safety
    ///
    /// The caller must ensure `path` points to a trusted, ABI-compatible `nuvio_engine`
    /// shared library whose initialization routines are safe to execute on load.
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        if !p.is_file() {
            return Err(P2pError::LibraryNotFound);
        }
        let lib = unsafe { Library::new(p)? };
        Ok(Self { lib })
    }

    pub fn api_version(&self) -> Result<u32> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> u32> =
                self.lib.get(b"nuvio_engine_api_version\0")?;
            Ok(func())
        }
    }

    pub fn protocol_backend_version(&self) -> Result<String> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *const c_char> =
                self.lib.get(b"nuvio_engine_protocol_backend_version\0")?;
            let ptr = func();
            if ptr.is_null() {
                return Ok(String::new());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    pub fn has_symbol(&self, symbol: &[u8]) -> bool {
        unsafe { self.lib.get::<*const ()>(symbol).is_ok() }
    }
}
