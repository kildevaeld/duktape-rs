use super::duktape_ffi as duktape;
use super::duktape_ffi::*;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

#[inline(always)]
pub unsafe fn duk_create_heap_default() -> *mut duktape::duk_context {
    duktape::duk_create_heap(None, None, None, ptr::null_mut(), None)
}

#[inline(always)]
pub unsafe fn duk_safe_to_string(
    ctx: *mut duktape::duk_context,
    index: duktape::duk_idx_t,
) -> *const c_char {
    duktape::duk_safe_to_lstring(ctx, index, ptr::null_mut())
}

/* PLAIN */
#[inline(always)]
pub unsafe fn duk_eval(ctx: *mut duktape::duk_context) {
    duktape::duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_eval_noresult(ctx: *mut duktape::duk_context) {
    duktape::duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_NORESULT | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_peval(ctx: *mut duktape::duk_context) -> duktape::duk_int_t {
    duktape::duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_SAFE | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_peval_noresult(ctx: *mut duktape::duk_context) -> duktape::duk_int_t {
    duktape::duk_eval_raw(
        ctx,
        ptr::null(),
        0,
        1 | DUK_COMPILE_EVAL | DUK_COMPILE_SAFE | DUK_COMPILE_NORESULT | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_compile(ctx: *mut duktape::duk_context, flags: duktape::duk_uint_t) {
    duktape::duk_compile_raw(ctx, ptr::null(), 0, 2 | flags);
}

#[inline(always)]
pub unsafe fn duk_pcompile(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
) -> duktape::duk_int_t {
    duktape::duk_compile_raw(ctx, ptr::null(), 0, 2 | flags | DUK_COMPILE_SAFE)
}

/* STRING */

pub unsafe fn duk_eval_string(ctx: *mut duktape::duk_context, src: *const c_char) {
    duktape::duk_eval_raw(
        ctx,
        src,
        0,
        0 | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_eval_string_noresult(ctx: *mut duktape::duk_context, src: *const c_char) {
    duktape::duk_eval_raw(
        ctx,
        src,
        0,
        0 | DUK_COMPILE_EVAL
            | DUK_COMPILE_NOSOURCE
            | DUK_COMPILE_STRLEN
            | DUK_COMPILE_NORESULT
            | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_peval_string_noresult(
    ctx: *mut duktape::duk_context,
    src: *const c_char,
) -> duktape::duk_int_t {
    duktape::duk_eval_raw(
        ctx,
        src,
        0,
        0 | DUK_COMPILE_EVAL
            | DUK_COMPILE_SAFE
            | DUK_COMPILE_NOSOURCE
            | DUK_COMPILE_STRLEN
            | DUK_COMPILE_NORESULT
            | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_compile_string(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    src: *const c_char,
) {
    duktape::duk_compile_raw(
        ctx,
        src,
        0,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_compile_string_filename(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    src: *const c_char,
) {
    duktape::duk_compile_raw(
        ctx,
        src,
        0,
        1 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN,
    );
}

#[inline(always)]
pub unsafe fn duk_pcompile_string(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    src: *const c_char,
) -> duktape::duk_int_t {
    duktape::duk_compile_raw(
        ctx,
        src,
        0,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_pcompile_string_filename(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    src: *const c_char,
) -> duktape::duk_int_t {
    duktape::duk_compile_raw(
        ctx,
        src,
        0,
        1 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN,
    )
}

/* LSTRING */

#[inline(always)]
pub unsafe fn duk_eval_lstring(
    ctx: *mut duktape::duk_context,
    buf: *const c_char,
    len: duktape::duk_size_t,
) {
    duktape::duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_eval_lstring_noresult(
    ctx: *mut duktape::duk_context,
    buf: *const c_char,
    len: duktape::duk_size_t,
) {
    duktape::duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NORESULT | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_peval_lstring(
    ctx: *mut duktape::duk_context,
    buf: *const c_char,
    len: duktape::duk_size_t,
) -> duktape::duk_int_t {
    duktape::duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_SAFE | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_peval_lstring_noresult(
    ctx: *mut duktape::duk_context,
    buf: *const c_char,
    len: duktape::duk_size_t,
) -> duktape::duk_int_t {
    duktape::duk_eval_raw(
        ctx,
        buf,
        len,
        0 | DUK_COMPILE_SAFE
            | DUK_COMPILE_EVAL
            | DUK_COMPILE_NOSOURCE
            | DUK_COMPILE_NORESULT
            | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_compile_lstring(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    buf: *const c_char,
    len: duktape::duk_size_t,
) {
    duktape::duk_compile_raw(
        ctx,
        buf,
        len,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    );
}

#[inline(always)]
pub unsafe fn duk_compile_lstring_filename(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    buf: *const c_char,
    len: duktape::duk_size_t,
) {
    duktape::duk_compile_raw(ctx, buf, len, 1 | flags | DUK_COMPILE_NOSOURCE);
}

#[inline(always)]
pub unsafe fn duk_pcompile_lstring(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    buf: *const c_char,
    len: duktape::duk_size_t,
) -> duktape::duk_int_t {
    duktape::duk_compile_raw(
        ctx,
        buf,
        len,
        0 | flags | DUK_COMPILE_NOSOURCE | DUK_COMPILE_NOFILENAME,
    )
}

#[inline(always)]
pub unsafe fn duk_pcompile_lstring_filename(
    ctx: *mut duktape::duk_context,
    flags: duktape::duk_uint_t,
    buf: *const c_char,
    len: duktape::duk_size_t,
) -> duktape::duk_int_t {
    duktape::duk_compile_raw(ctx, buf, len, 1 | flags | DUK_COMPILE_NOSOURCE)
}

pub unsafe fn duk_dump_context_stdout(ctx: *mut duktape::duk_context) {
    duk_push_context_dump(ctx);
    let ostr = duk_get_string(ctx, -1);
    let s = CStr::from_ptr(ostr).to_str().unwrap().to_string();
    duk_pop(ctx);
    println!("{}", s);
}
