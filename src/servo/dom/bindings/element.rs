use js::rust::{bare_compartment, methods, jsobj};
use js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL,
            JS_THIS_OBJECT, JS_SET_RVAL, JSPROP_NATIVE_ACCESSORS};
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, jsid, JSClass, JSFreeOp, JSPropertySpec};
use js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
                            JS_DefineFunctions, JS_DefineProperty};
use js::jsapi::bindgen::*;
use js::glue::bindgen::*;
use js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub};

use content::content_task::{Content, task_from_context};
use layout::layout_task;
use dom::node::{Node, NodeScope, Element};
use dom::element::*;
use dom::bindings::node::NodeBundle;
use dom::bindings::utils::{rust_box, squirrel_away_unique, get_compartment, domstring_to_jsval};
use dom::bindings::utils::{str};
use libc::c_uint;
use ptr::null;
use dom::bindings::node::unwrap;


extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("element finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _node: ~NodeBundle = cast::reinterpret_cast(&RUST_JSVAL_TO_PRIVATE(val));
    }
}

pub fn init(compartment: &bare_compartment) {
    let obj = utils::define_empty_prototype(~"Element", Some(~"Node"), compartment);
    let attrs = @~[
        {name: compartment.add_name(~"tagName"),
         tinyid: 0,
         flags: (JSPROP_ENUMERATE | JSPROP_SHARED | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: {op: getTagName, info: null()},
         setter: {op: null(), info: null()}}];
    vec::push(&mut compartment.global_props, attrs);
    vec::as_imm_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });

    compartment.register_class(utils::instance_jsclass(~"GenericElementInstance",
                                                       finalize));

    let _ = utils::define_empty_prototype(~"HTMLElement", Some(~"Element"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLDivElement", Some(~"HTMLElement"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLScriptElement", Some(~"HTMLElement"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLHeadElement", Some(~"HTMLElement"), compartment);

    let obj = utils::define_empty_prototype(~"HTMLImageElement", Some(~"HTMLElement"), compartment);
    let attrs = @~[
        {name: compartment.add_name(~"width"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: {op: HTMLImageElement_getWidth, info: null()},
         setter: {op: HTMLImageElement_setWidth, info: null()}}];
    vec::push(&mut compartment.global_props, attrs);
    vec::as_imm_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });
}

#[allow(non_implicitly_copyable_typarams)]
extern fn HTMLImageElement_getWidth(cx: *JSContext, _argc: c_uint, vp: *mut JSVal)
    -> JSBool unsafe {
    let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
    if obj.is_null() {
        return 0;
    }

    let bundle = unwrap(obj);
    let node = (*bundle).payload.node;
    let scope = (*bundle).payload.scope;
    let width = scope.write(&node, |nd| {
        match nd.kind {
            ~Element(ed) => {
                match ed.kind {
                    ~HTMLImageElement(*) => {
                        let content = task_from_context(cx);
                        match (*content).query_layout(layout_task::ContentBox(node)) {
                            Ok(rect) => rect.width,
                            Err(()) => 0,
                        }
                        // TODO: if nothing is being rendered(?), return zero dimensions
                    }
                    _ => fail ~"why is this not an image element?"
                }
            },
            _ => fail ~"why is this not an element?"
        }
    });
    *vp = RUST_INT_TO_JSVAL(
              (width & (i32::max_value as int)) as libc::c_int);
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
extern fn HTMLImageElement_setWidth(cx: *JSContext, _argc: c_uint, vp: *mut JSVal)
    -> JSBool unsafe {
    let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
    if obj.is_null() {
        return 0;
    }

    let bundle = unwrap(obj);
    do (*bundle).payload.scope.write(&(*bundle).payload.node) |nd| {
        match nd.kind {
          ~Element(ref ed) => {
            match ed.kind {
              ~HTMLImageElement(*) => {
                let arg = ptr::offset(JS_ARGV(cx, cast::reinterpret_cast(&vp)), 0);
                ed.set_attr(~"width", int::str(RUST_JSVAL_TO_INT(*arg) as int))
              },
              _ => fail ~"why is this not an image element?"
            }
          }
          _ => fail ~"why is this not an element?"
        }
    };
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getTagName(cx: *JSContext, _argc: c_uint, vp: *mut JSVal)
    -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let bundle = unwrap(obj);
        do (*bundle).payload.scope.write(&(*bundle).payload.node) |nd| {
            match nd.kind {
              ~Element(ref ed) => {
                let s = str(copy ed.tag_name);
                *vp = domstring_to_jsval(cx, &s);
              }
              _ => {
                //XXXjdm should probably read the spec to figure out what to do here
                *vp = JSVAL_NULL;
              }
            }
        };
    }
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
pub fn create(cx: *JSContext, node: Node, scope: NodeScope) -> jsobj unsafe {
    let proto = scope.write(&node, |nd| {
        match nd.kind {
          ~Element(ed) => {
            match ed.kind {
              ~HTMLDivElement(*) => ~"HTMLDivElement",
              ~HTMLHeadElement(*) => ~"HTMLHeadElement",
              ~HTMLImageElement(*) => ~"HTMLImageElement",
              ~HTMLScriptElement(*) => ~"HTMLScriptElement",
              _ => ~"HTMLElement"
            }
          }
          _ => fail ~"element::create only handles elements"
        }
    });   

    //XXXjdm the parent should probably be the node parent instead of the global
    //TODO error checking
    let compartment = utils::get_compartment(cx);
    let obj = result::unwrap(
        compartment.new_object_with_proto(~"GenericElementInstance", move proto,
                                          compartment.global_obj.ptr));
 
    unsafe {
        let raw_ptr: *libc::c_void =
            cast::reinterpret_cast(&squirrel_away_unique(~NodeBundle(node, scope)));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }
    return obj;
}
