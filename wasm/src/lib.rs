extern crate wasm_bindgen;

// for web browser
use parser::ast::*;
use parser::md_parser::md_parse;

use wasm_bindgen::prelude::*;

// for other webassembly env
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};

#[wasm_bindgen]
pub fn parse_markdown(source: &str) -> String {
    let mut node = ASTNode::new(ASTElm {
        ..Default::default()
    });
    node = md_parse(source, node);
    let serialized = serde_json::to_string(&node).unwrap();
    serialized
}

// for other webassembly env
// Low Level API
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut c_void {
    let mut buffer = Vec::with_capacity(size);
    let pointer = buffer.as_mut_ptr();
    mem::forget(buffer);

    pointer as *mut c_void
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, capacity);
    }
}

#[no_mangle]
pub extern "C" fn deallocate_str( ptr: *mut c_char ) {
    // retake pointer to free memory
    unsafe { let _ = CString::from_raw( ptr ); }
}

fn string_safe(str_ptr: *mut c_char) -> String {
    unsafe { CStr::from_ptr(str_ptr).to_string_lossy().to_owned().to_string() }
}

#[no_mangle]
pub extern "C" fn ffi_parse_markdown(source_ptr: *mut c_char) -> *mut c_char {
    let source = string_safe( source_ptr );
    let result = parse_markdown( &source );
    CString::new( result ).expect("CString::new failed").into_raw()
}
