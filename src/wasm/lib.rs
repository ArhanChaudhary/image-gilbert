use js_sys::Array;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, PointerEvent, Worker,
    WorkerOptions, WorkerType,
};

mod gilbert;
mod handlers;
mod renderer;
mod utils;
mod worker;

#[wasm_bindgen(start)]
fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = runMain)]
pub fn run_main() {
    let document = web_sys::window().unwrap().document().unwrap();

    let ctx = Rc::new(
        utils::get_element_by_id::<HtmlCanvasElement>(&document, "canvas")
            .get_context_with_context_options(
                "2d",
                &serde_wasm_bindgen::to_value(&CanvasContextOptions {
                    desynchronized: false,
                })
                .unwrap(),
            )
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap(),
    );
    let upload_input = Rc::new(utils::get_element_by_id::<HtmlInputElement>(
        &document, "upload",
    ));
    let start_input = Rc::new(utils::get_element_by_id::<HtmlInputElement>(
        &document, "start",
    ));
    let step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "step");
    let stop_input = utils::get_element_by_id::<HtmlInputElement>(&document, "stop");
    let change_speed_input =
        utils::get_element_by_id::<HtmlInputElement>(&document, "change-speed");
    let change_step_input = utils::get_element_by_id::<HtmlInputElement>(&document, "change-step");

    let worker_options = WorkerOptions::new();
    worker_options.set_type(WorkerType::Module);
    let worker = Rc::new(Worker::new_with_options("./worker.js", &worker_options).unwrap());

    let worker_message = Array::new();
    worker_message.push(&wasm_bindgen::module());
    worker_message.push(&wasm_bindgen::memory());
    worker.post_message(&worker_message).unwrap();

    {
        let upload_input_clone = upload_input.clone();
        let ctx_clone = ctx.clone();
        let onchange_closure = Closure::<dyn Fn()>::new(move || {
            let upload_input_clone = upload_input_clone.clone();
            let ctx_clone = ctx_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                handlers::uploaded_image(upload_input_clone, ctx_clone).await;
            });
        });
        upload_input.set_onchange(Some(onchange_closure.as_ref().unchecked_ref()));
        onchange_closure.forget();
    }

    let raf_handle = Rc::new(RefCell::new(None));
    {
        let worker_clone = worker.clone();
        let ctx_clone = ctx.clone();
        let raf_handle_clone = raf_handle.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            handlers::clicked_start(
                ctx_clone.clone(),
                worker_clone.clone(),
                raf_handle_clone.clone(),
            );
        });
        start_input.set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
        onclick_closure.forget();
    }

    {
        let ctx_clone = ctx.clone();
        let raf_handle_clone = raf_handle.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            handlers::clicked_stop(ctx_clone.clone(), raf_handle_clone.clone());
        });
        stop_input.set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
        onclick_closure.forget();
    }

    {
        let worker_clone = worker.clone();
        let ctx_clone = ctx.clone();
        let raf_handle_clone = raf_handle.clone();
        let onclick_closure = Closure::<dyn Fn()>::new(move || {
            let ctx_clone = ctx_clone.clone();
            let worker_clone = worker_clone.clone();
            let raf_handle_clone = raf_handle_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                handlers::clicked_step(ctx_clone.clone(), worker_clone.clone(), raf_handle_clone.clone()).await;
            });
        });
        step_input.set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
        onclick_closure.forget();
    }

    {
        let oninput_closure = Closure::<dyn Fn(_)>::new(move |e: PointerEvent| {
            handlers::inputted_speed(e);
        });
        change_speed_input.set_oninput(Some(oninput_closure.as_ref().unchecked_ref()));
        oninput_closure.forget();
    }

    {
        let oninput_closure = Closure::<dyn Fn(_)>::new(move |e: PointerEvent| {
            handlers::inputted_step(e);
        });
        change_step_input.set_oninput(Some(oninput_closure.as_ref().unchecked_ref()));
        oninput_closure.forget();
    }
}

#[derive(Serialize)]
struct CanvasContextOptions {
    desynchronized: bool,
}
