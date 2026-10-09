// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::ffi::CString;

use super::types::{LoreString, LoreStringArray, MAX_COPIED_STRING};
use crate::error::{Error, Result};

/// UTF-8 text kept alive for the duration of one library call.
///
/// An empty string is a null pointer and a length of zero, which is what
/// `lore.h` requires. The library copies argument bytes before the call
/// returns, so this value only has to outlive that return.
pub struct Text {
    storage: Option<CString>,
}

impl Text {
    pub fn new(field: &'static str, value: &str) -> Result<Self> {
        if value.is_empty() {
            return Ok(Self { storage: None });
        }
        if value.len() > MAX_COPIED_STRING {
            return Err(Error::InvalidArgument {
                field,
                message: format!("longer than {MAX_COPIED_STRING} bytes"),
            });
        }
        let storage = CString::new(value).map_err(|_| Error::InvalidArgument {
            field,
            message: "contains a NUL byte".to_string(),
        })?;
        Ok(Self {
            storage: Some(storage),
        })
    }

    pub fn as_ffi(&self) -> LoreString {
        match &self.storage {
            Some(storage) => LoreString {
                ptr: storage.as_ptr().cast(),
                length: storage.as_bytes().len(),
            },
            None => LoreString {
                ptr: std::ptr::null(),
                length: 0,
            },
        }
    }
}

pub struct TextList {
    items: Vec<Text>,
    view: Vec<LoreString>,
}

impl TextList {
    pub fn new(field: &'static str, values: &[&str]) -> Result<Self> {
        let items = values
            .iter()
            .map(|value| Text::new(field, value))
            .collect::<Result<Vec<_>>>()?;
        let view = items.iter().map(Text::as_ffi).collect();
        Ok(Self { items, view })
    }

    pub fn as_ffi(&self) -> LoreStringArray {
        debug_assert_eq!(self.items.len(), self.view.len());
        LoreStringArray {
            ptr: if self.view.is_empty() {
                std::ptr::null()
            } else {
                self.view.as_ptr()
            },
            count: self.view.len(),
        }
    }
}

/// Copy a string out of an event. The pointer is only valid until the
/// callback returns, so this owns the bytes.
///
/// # Safety
///
/// `value` must be a `lore_string_t` the library handed to the current
/// callback invocation. A null pointer with length zero is empty. A non-zero
/// length must address that many readable bytes.
pub unsafe fn copy_text(value: LoreString) -> Result<String> {
    if value.length == 0 {
        return Ok(String::new());
    }
    if value.ptr.is_null() || value.length > MAX_COPIED_STRING {
        return Err(Error::Malformed {
            operation: "event",
            message: "string pointer or length is not usable".to_string(),
        });
    }
    // Safety: the caller guarantees the library still owns this buffer and
    // that `length` is its byte count, excluding any trailing NUL.
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.length) };
    String::from_utf8(bytes.to_vec()).map_err(|_| Error::Malformed {
        operation: "event",
        message: "string is not UTF-8".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_a_null_pointer() {
        let text = Text::new("field", "").unwrap();
        let ffi = text.as_ffi();
        assert!(ffi.ptr.is_null());
        assert_eq!(ffi.length, 0);
    }

    #[test]
    fn interior_nul_is_rejected() {
        let error = match Text::new("field", "a\0b") {
            Ok(_) => panic!("interior NUL must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            Error::InvalidArgument { field: "field", .. }
        ));
    }
}
