// SPDX-License-Identifier: AGPL-3.0-only
//! Main-thread bootstrap (wasm32 only): spawn the data worker, forward
//! configuration and visibility, log received view updates, and show the
//! "reload required" banner on a protocol mismatch. Components arrive in
//! Phase 3.

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Array, ArrayBuffer, Uint8Array};
use muconsole_layout::{SafeArea, Viewport};
use muconsole_protocol::{UiCommand, WorkerConfig};
use muconsole_theme::wireframe::WireframeSkin;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, MessageEvent, Worker};

use crate::client::{reload_banner_text, ClientUpdate, DataClient, WORKER_URL};
use crate::components::{render_console, render_html_shell};
use crate::state::ConsoleState;

thread_local! {
    static WORKER: RefCell<Option<Worker>> = const { RefCell::new(None) };
}

/// Post a command as a transferred `ArrayBuffer`.
fn send(worker: &Worker, cmd: &UiCommand) {
    if let Ok(bytes) = DataClient::encode_command(cmd) {
        let buf = Uint8Array::from(bytes.as_slice()).buffer();
        let _ = worker.post_message_with_transfer(&buf, &Array::of1(&buf));
    }
}

fn show_banner(text: &str) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let el = doc.get_element_by_id("muconsole-reload").or_else(|| {
        let el = doc.create_element("div").ok()?;
        el.set_id("muconsole-reload");
        let _ = el.set_attribute("role", "alert");
        let _ = el.set_attribute(
            "style",
            "position:fixed;inset:0 0 auto 0;padding:12px 16px;font:16px system-ui;\
             background:#ffb43a;color:#060402;z-index:10",
        );
        doc.body()?.append_child(&el).ok()?;
        Some(el)
    });
    if let Some(el) = el {
        el.set_text_content(Some(text));
    }
}

/// Spawn the worker and wire the message bridge.
pub fn start() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let location = window.location();
    let cfg = WorkerConfig::from_query(
        &location.search().unwrap_or_default(),
        &location.origin().unwrap_or_default(),
    );
    let worker = match Worker::new(WORKER_URL) {
        Ok(w) => w,
        Err(e) => {
            console::error_2(&JsValue::from_str("muconsole: worker spawn failed"), &e);
            return;
        }
    };
    let state = Rc::new(RefCell::new(ConsoleState::new()));
    {
        let hash = location.hash().unwrap_or_default();
        let search = location.search().unwrap_or_default();
        state.borrow_mut().init_from_env(&hash, &search, None);
    }
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |ev: MessageEvent| {
        let data = ev.data();
        let bytes = data
            .dyn_ref::<ArrayBuffer>()
            .map(|b| Uint8Array::new(b).to_vec())
            .unwrap_or_default();
        let mut st = state.borrow_mut();
        match st.on_worker_frame(&bytes) {
            ClientUpdate::ReloadRequired { expected, found } => {
                show_banner(&reload_banner_text(expected, found));
            }
            ClientUpdate::View => {
                if let Some(win) = web_sys::window() {
                    let w = win
                        .inner_width()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1512.0);
                    let h = win
                        .inner_height()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(860.0);
                    let svg =
                        render_console(&WireframeSkin, &st, Viewport::new(w, h), SafeArea::ZERO);
                    let html = render_html_shell(&svg, &st);
                    if let Some(body) = win.document().and_then(|d| d.body()) {
                        body.set_inner_html(&html);
                    }
                }
                console::log_1(&JsValue::from_str(&st.client.summary()));
            }
            ClientUpdate::Heartbeat => {}
            other => console::log_1(&JsValue::from_str(&format!("muconsole: {other:?}"))),
        }
    });
    worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    send(&worker, &UiCommand::Configure(cfg));

    if let Some(doc) = window.document() {
        let w = worker.clone();
        let on_visibility = Closure::<dyn FnMut()>::new(move || {
            let hidden = web_sys::window()
                .and_then(|w| w.document())
                .is_some_and(|d| d.hidden());
            send(&w, &UiCommand::SetVisibility { hidden });
        });
        let _ = doc.add_event_listener_with_callback(
            "visibilitychange",
            on_visibility.as_ref().unchecked_ref(),
        );
        on_visibility.forget();
    }
    WORKER.with(|slot| *slot.borrow_mut() = Some(worker));
}
