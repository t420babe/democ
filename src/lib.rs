use nalgebra_glm;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
  console, AudioContext, Document, Window, EventTarget, HtmlCanvasElement, HtmlMediaElement, WebGl2RenderingContext,
  WebGlBuffer,
  AnalyserNode,
};

mod buffer_attrib;
mod buffers;
mod program_info;
mod shaders;
mod utils;
mod web;
use crate::{buffer_attrib::BufferAttrib, program_info::ProgramInfo, utils::*};

/// Get Window in context
pub fn window() -> Window {
  web_sys::window().expect("Error. `window` is not in this context.")
}

/// Get Document in Window
pub fn document() -> Document {
  window().document().expect("Error. `document` is not in this `window`.")
}

/// Get Canvas in Document
pub fn canvas() -> Result<HtmlCanvasElement, JsValue> {
  let canvas = document().get_element_by_id("canvas").expect("Error. `canvas` is not in this `document`.");

  Ok(canvas.dyn_into::<HtmlCanvasElement>()?)
}

pub fn request_animation_frame(f: &Closure<dyn FnMut(f32)>) {
  window()
    .request_animation_frame(f.as_ref().unchecked_ref())
    .expect("Error. Did not register `RequestAnimationFrame`");
}

fn vec_to_js_array(vec: Vec<u8>) -> js_sys::Array {
  vec.into_iter().map(JsValue::from).collect()
}

async fn audio() -> Result<(), JsValue> {
  let context = web_sys::AudioContext::new()?;
  let node = context.create_analyser()?;

  let navigator = &window().navigator();
  let mut media_stream_constraints = web_sys::MediaStreamConstraints::new();
  &media_stream_constraints.audio(&JsValue::TRUE);
  &media_stream_constraints.video(&JsValue::FALSE);
  let stream_promise = navigator.media_devices()?.get_user_media_with_constraints(&media_stream_constraints)?;
  let stream: web_sys::MediaStream = JsFuture::from(stream_promise).await?.dyn_into()?;

  // Buffer to hold fft data
  let kMaxFrequency = 20000;
  let sample_rate = context.sample_rate() as u32;
  let fft_size = node.fft_size() / 2;
  let buffer_size = (kMaxFrequency / sample_rate * fft_size) as usize;
  let buffer_size: usize = 16;
  let mut buffer = vec![0; buffer_size];
  let arr = js_sys::Array::new();
  &arr.set(4, JsValue::from_f64(1.0));


  let audio_node = &context.create_media_stream_source(&stream)?;
  audio_node.connect_with_audio_node(&node)?;


  // Draw scene every 0.01 seconds
  let ref_count = Rc::new(RefCell::new(None));
  let ref_count_clone = ref_count.clone();

  *ref_count_clone.borrow_mut() = Some(Closure::wrap(Box::new(move |t| {
    let buf = buffer.clone();
    draw_loop(&node, buf);
    request_animation_frame(ref_count.borrow().as_ref().unwrap());
  }) as Box<dyn FnMut(f32)>));

  request_animation_frame(ref_count_clone.borrow().as_ref().unwrap());

  Ok(())
}

/// Audio draw loop
fn draw_loop(node: &AnalyserNode, mut buffer: Vec<u8>) -> Result<(), JsValue> {
  &node.get_byte_frequency_data(&mut buffer);
  web_sys::console::log(&vec_to_js_array(buffer));
  Ok(())
}


#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
  let document = document();
  let canvas = canvas()?;

  let audio_context = web_sys::AudioContext::new()?;
  let gl_context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

  web::run(audio_context, gl_context).await?;


  // shaders::do_webgl(gl_context)?;
  // audio().await?;

  Ok(())
}
