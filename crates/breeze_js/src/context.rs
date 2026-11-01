use crate::events::EventManager;
use crate::timer::TimerQueue;
use boa_engine::native_function::NativeFunction;
use boa_engine::{Context, JsArgs, JsObject, JsString, JsValue, Source};
use breeze_html::dom::{Document, Node, NodeData, NodeRef};
use breeze_net::{HttpClient, Origin};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

pub struct JsRuntime {
    pub context: Context,
    pub doc: Document,
    pub event_manager: Rc<RefCell<EventManager>>,
    pub timer_queue: Rc<RefCell<TimerQueue>>,
    pub base_url: Option<Url>,
    pub current_origin: Option<Origin>,
    pub client: HttpClient,
    pub console_logs: Rc<RefCell<Vec<String>>>,
    pub local_storage: Rc<RefCell<HashMap<String, String>>>,
}

impl JsRuntime {
    pub fn new(doc: Document, base_url: Option<Url>) -> Self {
        let context = Context::default();
        let event_manager = Rc::new(RefCell::new(EventManager::new()));
        let timer_queue = Rc::new(RefCell::new(TimerQueue::new()));
        let console_logs = Rc::new(RefCell::new(Vec::new()));
        let local_storage = Rc::new(RefCell::new(HashMap::new()));
        let current_origin = base_url.as_ref().and_then(Origin::from_url);

        let mut runtime = Self {
            context,
            doc,
            event_manager,
            timer_queue,
            base_url,
            current_origin,
            client: HttpClient::new(),
            console_logs,
            local_storage,
        };

        runtime.init_globals();
        runtime
    }

    fn init_globals(&mut self) {
        // 1. console.log / console.error
        let console_obj = JsObject::with_object_proto(self.context.intrinsics());
        let logs_clone = Rc::clone(&self.console_logs);
        let log_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let msg = args
                    .iter()
                    .map(|v| v.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("[JS console.log] {}", msg);
                logs_clone.borrow_mut().push(msg);
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());

        let logs_clone2 = Rc::clone(&self.console_logs);
        let err_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let msg = args
                    .iter()
                    .map(|v| v.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(" ");
                eprintln!("[JS console.error] {}", msg);
                logs_clone2.borrow_mut().push(format!("ERROR: {}", msg));
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());

        console_obj.set(JsString::from("log"), log_fn, true, &mut self.context).unwrap();
        console_obj.set(JsString::from("error"), err_fn, true, &mut self.context).unwrap();
        self.context.global_object().set(JsString::from("console"), console_obj, true, &mut self.context).unwrap();

        // 2. setTimeout & setInterval & clearTimeout & clearInterval
        let tq_clone = Rc::clone(&self.timer_queue);
        let set_timeout_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, _ctx| {
                let cb = args.get_or_undefined(0).clone();
                let delay = args.get_or_undefined(1).to_number(_ctx).unwrap_or(0.0).max(0.0) as u64;
                let id = tq_clone.borrow_mut().set_timeout(cb, delay);
                Ok(JsValue::from(id))
            })
        }.to_js_function(self.context.realm());

        let tq_clone2 = Rc::clone(&self.timer_queue);
        let set_interval_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, _ctx| {
                let cb = args.get_or_undefined(0).clone();
                let delay = args.get_or_undefined(1).to_number(_ctx).unwrap_or(0.0).max(0.0) as u64;
                let id = tq_clone2.borrow_mut().set_interval(cb, delay);
                Ok(JsValue::from(id))
            })
        }.to_js_function(self.context.realm());

        let tq_clone3 = Rc::clone(&self.timer_queue);
        let clear_timeout_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, _ctx| {
                let id = args.get_or_undefined(0).to_number(_ctx).unwrap_or(0.0) as u32;
                tq_clone3.borrow_mut().clear(id);
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());

        let tq_clone4 = Rc::clone(&self.timer_queue);
        let clear_interval_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, _ctx| {
                let id = args.get_or_undefined(0).to_number(_ctx).unwrap_or(0.0) as u32;
                tq_clone4.borrow_mut().clear(id);
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());

        self.context.global_object().set(JsString::from("setTimeout"), set_timeout_fn, true, &mut self.context).unwrap();
        self.context.global_object().set(JsString::from("setInterval"), set_interval_fn, true, &mut self.context).unwrap();
        self.context.global_object().set(JsString::from("clearTimeout"), clear_timeout_fn, true, &mut self.context).unwrap();
        self.context.global_object().set(JsString::from("clearInterval"), clear_interval_fn, true, &mut self.context).unwrap();

        // 3. localStorage
        let ls_obj = JsObject::with_object_proto(self.context.intrinsics());
        let store_clone1 = Rc::clone(&self.local_storage);
        let get_item_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let key = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                let store = store_clone1.borrow();
                let val = store.get(&key).cloned();
                match val {
                    Some(s) => Ok(JsValue::from(JsString::from(s))),
                    None => Ok(JsValue::null()),
                }
            })
        }.to_js_function(self.context.realm());

        let store_clone2 = Rc::clone(&self.local_storage);
        let set_item_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let key = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                let val = args.get_or_undefined(1).to_string(ctx)?.to_std_string_escaped();
                store_clone2.borrow_mut().insert(key, val);
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());

        ls_obj.set(JsString::from("getItem"), get_item_fn, true, &mut self.context).unwrap();
        ls_obj.set(JsString::from("setItem"), set_item_fn, true, &mut self.context).unwrap();
        self.context.global_object().set(JsString::from("localStorage"), ls_obj, true, &mut self.context).unwrap();

        // 4. location
        let loc_obj = JsObject::with_object_proto(self.context.intrinsics());
        let href = self.base_url.as_ref().map(|u| u.as_str()).unwrap_or("about:blank");
        loc_obj.set(JsString::from("href"), JsString::from(href), true, &mut self.context).unwrap();
        loc_obj.set(
            JsString::from("origin"),
            JsString::from(self.current_origin.as_ref().map(|o| format!("{}://{}:{}", o.scheme, o.host, o.port)).unwrap_or_default()),
            true,
            &mut self.context,
        ).unwrap();
        self.context.global_object().set(JsString::from("location"), loc_obj, true, &mut self.context).unwrap();

        // 5. window (points to global)
        let global_val = self.context.global_object();
        self.context.global_object().set(JsString::from("window"), global_val, true, &mut self.context).unwrap();

        // 6. window.addEventListener
        let event_mgr1 = Rc::clone(&self.event_manager);
        let add_event_listener_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let event_type = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                if let Some(cb_obj) = args.get_or_undefined(1).as_object() {
                    event_mgr1.borrow_mut().add_event_listener("window", &event_type, cb_obj.clone());
                }
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());
        self.context.global_object().set(JsString::from("addEventListener"), add_event_listener_fn, true, &mut self.context).unwrap();

        // 7. fetch(url)
        let base_url_clone = self.base_url.clone();
        let curr_orig_clone = self.current_origin.clone();
        let fetch_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let url_str = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                let target_url = if let Ok(u) = url::Url::parse(&url_str) {
                    Some(u)
                } else if let Some(base) = &base_url_clone {
                    base.join(&url_str).ok()
                } else {
                    None
                };

                if let Some(u) = target_url {
                    let client = breeze_net::HttpClient::new();
                    match client.fetch(&u, breeze_net::ResourceType::Fetch, curr_orig_clone.as_ref()) {
                        Ok(resp) => {
                            let resp_obj = JsObject::with_object_proto(ctx.intrinsics());
                            resp_obj.set(JsString::from("status"), JsValue::from(resp.status), true, ctx).unwrap();
                            resp_obj.set(JsString::from("ok"), JsValue::from(resp.status >= 200 && resp.status < 300), true, ctx).unwrap();

                            let body_text1 = resp.text();
                            let text_fn = NativeFunction::from_closure(move |_this, _args, ctx2| {
                                let p = boa_engine::object::builtins::JsPromise::resolve(
                                    JsValue::from(JsString::from(body_text1.as_str())),
                                    ctx2,
                                );
                                Ok(JsValue::from(p))
                            }).to_js_function(ctx.realm());

                            let body_text2 = resp.text();
                            let json_fn = NativeFunction::from_closure(move |_this, _args, ctx2| {
                                let expr = format!("({})", body_text2);
                                let source = Source::from_bytes(expr.as_bytes());
                                let val = ctx2.eval(source).unwrap_or(JsValue::null());
                                let p = boa_engine::object::builtins::JsPromise::resolve(val, ctx2);
                                Ok(JsValue::from(p))
                            }).to_js_function(ctx.realm());

                            resp_obj.set(JsString::from("text"), text_fn, true, ctx).unwrap();
                            resp_obj.set(JsString::from("json"), json_fn, true, ctx).unwrap();

                            let p = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(resp_obj), ctx);
                            return Ok(JsValue::from(p));
                        }
                        Err(e) => {
                            let err = boa_engine::JsNativeError::typ().with_message(e.to_string());
                            let p = boa_engine::object::builtins::JsPromise::reject(err, ctx);
                            return Ok(JsValue::from(p));
                        }
                    }
                }

                let err = boa_engine::JsNativeError::typ().with_message("Invalid URL");
                let p = boa_engine::object::builtins::JsPromise::reject(err, ctx);
                Ok(JsValue::from(p))
            })
        }.to_js_function(self.context.realm());
        self.context.global_object().set(JsString::from("fetch"), fetch_fn, true, &mut self.context).unwrap();

        // 8. document object
        self.bind_document();
    }

    fn bind_document(&mut self) {
        let doc_obj = JsObject::with_object_proto(self.context.intrinsics());
        let doc_root = Rc::clone(&self.doc.root);

        let root_for_id = Rc::clone(&doc_root);
        let get_element_by_id_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let id = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                let doc = Document { root: Rc::clone(&root_for_id) };
                if let Some(node) = doc.get_element_by_id(&id) {
                    Ok(create_element_js_object(ctx, node))
                } else {
                    Ok(JsValue::null())
                }
            })
        }.to_js_function(self.context.realm());

        let create_element_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let tag_name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                let elem = Node::new_element(&tag_name, HashMap::new());
                Ok(create_element_js_object(ctx, elem))
            })
        }.to_js_function(self.context.realm());

        // document.querySelector
        let root_for_qs = Rc::clone(&doc_root);
        let query_selector_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let sel = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                let doc = Document { root: Rc::clone(&root_for_qs) };
                if sel.starts_with('#') {
                    if let Some(node) = doc.get_element_by_id(&sel[1..]) {
                        return Ok(create_element_js_object(ctx, node));
                    }
                } else if sel.starts_with('.') {
                    let elems = doc.get_elements_by_class_name(&sel[1..]);
                    if let Some(node) = elems.first() {
                        return Ok(create_element_js_object(ctx, Rc::clone(node)));
                    }
                } else {
                    let elems = doc.get_elements_by_tag_name(&sel);
                    if let Some(node) = elems.first() {
                        return Ok(create_element_js_object(ctx, Rc::clone(node)));
                    }
                }
                Ok(JsValue::null())
            })
        }.to_js_function(self.context.realm());

        // document.addEventListener
        let event_mgr2 = Rc::clone(&self.event_manager);
        let doc_add_event_listener_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let event_type = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
                if let Some(cb_obj) = args.get_or_undefined(1).as_object() {
                    event_mgr2.borrow_mut().add_event_listener("document", &event_type, cb_obj.clone());
                }
                Ok(JsValue::undefined())
            })
        }.to_js_function(self.context.realm());

        doc_obj.set(JsString::from("getElementById"), get_element_by_id_fn, true, &mut self.context).unwrap();
        doc_obj.set(JsString::from("createElement"), create_element_fn, true, &mut self.context).unwrap();
        doc_obj.set(JsString::from("querySelector"), query_selector_fn, true, &mut self.context).unwrap();
        doc_obj.set(JsString::from("addEventListener"), doc_add_event_listener_fn, true, &mut self.context).unwrap();

        let doc_clone = Document { root: Rc::clone(&doc_root) };
        let body_nodes = doc_clone.get_elements_by_tag_name("body");
        if let Some(body_node) = body_nodes.first() {
            let body_val = create_element_js_object(&mut self.context, Rc::clone(body_node));
            doc_obj.set(JsString::from("body"), body_val, true, &mut self.context).unwrap();
        }

        self.context.global_object().set(JsString::from("document"), doc_obj, true, &mut self.context).unwrap();
    }

    pub fn execute(&mut self, script: &str) -> Result<JsValue, String> {
        let source = Source::from_bytes(script.as_bytes());
        self.context.eval(source).map_err(|e| format!("{:?}", e))
    }

    pub fn tick_timers(&mut self) {
        let ready_timers = self.timer_queue.borrow_mut().collect_ready();
        for (_id, cb) in ready_timers {
            if let Some(obj) = cb.as_object() {
                let _ = obj.call(&JsValue::undefined(), &[], &mut self.context);
            }
        }
    }
}

pub fn create_element_js_object(context: &mut Context, node: NodeRef) -> JsValue {
    let obj = JsObject::with_object_proto(context.intrinsics());

    let text = Document::text_content(&node);
    obj.set(JsString::from("textContent"), JsString::from(text.as_str()), true, context).unwrap();
    obj.set(JsString::from("innerHTML"), JsString::from(text.as_str()), true, context).unwrap();

    let node_borrow = node.borrow();
    if let NodeData::Element(el) = &node_borrow.data {
        obj.set(JsString::from("tagName"), JsString::from(el.tag_name.to_ascii_uppercase()), true, context).unwrap();
        if let Some(id) = &el.id {
            obj.set(JsString::from("id"), JsString::from(id.as_str()), true, context).unwrap();
        }
        if let Some(val) = el.get_attribute("value") {
            obj.set(JsString::from("value"), JsString::from(val), true, context).unwrap();
        }
    }
    drop(node_borrow);

    let node_clone = Rc::clone(&node);
    let append_child_fn = unsafe {
        NativeFunction::from_closure(move |_this, args, ctx| {
            let child_arg = args.get_or_undefined(0);
            if let Some(child_obj) = child_arg.as_object() {
                if let Ok(content_val) = child_obj.get(JsString::from("textContent"), ctx) {
                    let text = content_val.to_string(ctx)?.to_std_string_escaped();
                    let child_node = Node::new_text(&text);
                    Document::append_child(&node_clone, &child_node);
                }
            }
            Ok(child_arg.clone())
        })
    }.to_js_function(context.realm());

    let node_clone2 = Rc::clone(&node);
    let set_attr_fn = unsafe {
        NativeFunction::from_closure(move |_this, args, ctx| {
            let name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
            let val = args.get_or_undefined(1).to_string(ctx)?.to_std_string_escaped();
            let mut nb = node_clone2.borrow_mut();
            if let NodeData::Element(el) = &mut nb.data {
                el.set_attribute(&name, &val);
            }
            Ok(JsValue::undefined())
        })
    }.to_js_function(context.realm());

    let node_clone3 = Rc::clone(&node);
    let get_attr_fn = unsafe {
        NativeFunction::from_closure(move |_this, args, ctx| {
            let name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
            let nb = node_clone3.borrow();
            if let NodeData::Element(el) = &nb.data {
                if let Some(val) = el.get_attribute(&name) {
                    return Ok(JsValue::from(JsString::from(val)));
                }
            }
            Ok(JsValue::null())
        })
    }.to_js_function(context.realm());

    // element.addEventListener
    let add_el_event_fn = unsafe {
        NativeFunction::from_closure(move |_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
    }.to_js_function(context.realm());

    obj.set(JsString::from("appendChild"), append_child_fn, true, context).unwrap();
    obj.set(JsString::from("setAttribute"), set_attr_fn, true, context).unwrap();
    obj.set(JsString::from("getAttribute"), get_attr_fn, true, context).unwrap();
    obj.set(JsString::from("addEventListener"), add_el_event_fn, true, context).unwrap();

    JsValue::from(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use breeze_html::parser::HtmlParser;

    #[test]
    fn test_js_execution_and_console() {
        let html = r#"<div id="greeting">Hello World</div>"#;
        let doc = HtmlParser::parse(html);
        let mut runtime = JsRuntime::new(doc, None);

        let script = r#"
            let a = 10;
            let b = 20;
            console.log("Calculated:", a + b);
            a + b;
        "#;
        let result = runtime.execute(script).unwrap();
        assert_eq!(result.to_number(&mut runtime.context).unwrap(), 30.0);

        let logs = runtime.console_logs.borrow();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], "Calculated: 30");
    }

    #[test]
    fn test_js_dom_manipulation() {
        let html = r#"<div id="target">Initial</div>"#;
        let doc = HtmlParser::parse(html);
        let mut runtime = JsRuntime::new(doc, None);

        let script = r#"
            let el = document.getElementById("target");
            el.setAttribute("data-status", "updated");
            let status = el.getAttribute("data-status");
            console.log("Status is:", status);
        "#;
        runtime.execute(script).unwrap();

        let elem = runtime.doc.get_element_by_id("target").unwrap();
        let status_val = {
            let nb = elem.borrow();
            if let NodeData::Element(el) = &nb.data {
                el.get_attribute("data-status").map(|s| s.to_string())
            } else {
                None
            }
        };
        assert_eq!(status_val, Some("updated".to_string()));
    }

    #[test]
    fn test_js_timers() {
        let doc = Document::new();
        let mut runtime = JsRuntime::new(doc, None);

        let script = r#"
            setTimeout(() => {
                console.log("Timer fired!");
            }, 0);
        "#;
        runtime.execute(script).unwrap();

        assert_eq!(runtime.console_logs.borrow().len(), 0);
        runtime.tick_timers();
        assert_eq!(runtime.console_logs.borrow().len(), 1);
        assert_eq!(runtime.console_logs.borrow()[0], "Timer fired!");
    }
}
