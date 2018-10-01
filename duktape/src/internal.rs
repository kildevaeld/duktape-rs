use duktape_sys::{self as duk, duk_context};

static REF_KEY: &'static [u8] = b"refs";

pub unsafe fn init_refs(ctx: *mut duk_context) {
    duk::duk_push_global_stash(ctx);
    if duk::duk_has_prop_lstring(ctx, -1, REF_KEY.as_ptr() as *const i8, 4) == 1 {
        duk::duk_pop(ctx);
        return;
    }
    duk::duk_push_array(ctx);
    duk::duk_push_int(ctx, 0);
    duk::duk_put_prop_index(ctx, -2, 0);
    duk::duk_put_prop_lstring(ctx, -2, REF_KEY.as_ptr() as *const i8, 4);
    duk::duk_pop(ctx);
}

unsafe fn get_refs(ctx: *mut duk_context) -> bool {
    duk::duk_push_global_stash(ctx);
    if duk::duk_has_prop_lstring(ctx, -1, REF_KEY.as_ptr() as *const i8, 4) == 0 {
        duk::duk_pop(ctx);
        return false;
    }

    duk::duk_get_prop_lstring(ctx, -1, REF_KEY.as_ptr() as *const i8, 4);

    duk::duk_remove(ctx, -2);

    true
}

pub unsafe fn make_ref(ctx: *mut duk_context) -> u32 {
    if duk::duk_is_undefined(ctx, -1) == 1 {
        duk::duk_pop(ctx);
        return 0;
    }
    // Get the "refs" array in the heap stash
    if !get_refs(ctx) {
        return 0;
    }

    // ref = refs[0]
    duk::duk_get_prop_index(ctx, -1, 0);
    let mut ret = duk::duk_get_int(ctx, -1);
    duk::duk_pop(ctx);

    // If there was a free slot, remove it from the list
    if ret != 0 {
        // refs[0] = refs[ref]
        duk::duk_get_prop_index(ctx, -1, ret as u32);
        duk::duk_put_prop_index(ctx, -2, 0);
    }
    // Otherwise use the end of the list
    else {
        // ref = refs.length;
        ret = duk::duk_get_length(ctx, -1) as i32;
    }

    // swap the array and the user value in the stack
    duk::duk_insert(ctx, -2);

    // refs[ref] = value
    duk::duk_put_prop_index(ctx, -2, ret as u32);

    // Remove the refs array from the stack.
    duk::duk_pop(ctx);

    return ret as u32;
}

pub unsafe fn push_ref(ctx: *mut duk_context, refer: u32) {
    if refer == 0 {
        duk::duk_push_undefined(ctx);
        return;
    }
    // Get the "refs" array in the heap stash
    if !get_refs(ctx) {
        return;
    }

    duk::duk_get_prop_index(ctx, -1, refer);

    duk::duk_remove(ctx, -2);
}

pub unsafe fn unref(ctx: *mut duk_context, refer: u32) {
    if refer == 0 {
        return;
    }
    // Get the "refs" array in the heap stash
    if !get_refs(ctx) {
        return;
    }

    // Insert a new link in the freelist

    // refs[ref] = refs[0]
    duk::duk_get_prop_index(ctx, -1, 0);
    duk::duk_put_prop_index(ctx, -2, refer);
    // refs[0] = ref
    duk::duk_push_int(ctx, refer as i32);
    duk::duk_put_prop_index(ctx, -2, 0);

    duk::duk_pop(ctx);
}
