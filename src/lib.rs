use nalgebra_glm;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
  console, AudioContext, EventTarget, HtmlCanvasElement, HtmlMediaElement, WebGl2RenderingContext,
  WebGlBuffer,
};

mod buffer_attrib;
mod buffers;
mod program_info;
mod shaders;
mod utils;
use crate::{buffer_attrib::BufferAttrib, program_info::ProgramInfo, utils::*};

pub fn window() -> web_sys::Window {
  web_sys::window().expect("Error. `window` is not in this context.")
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
  let document = window().document().expect("Error. `window` does not have a `document`.");
  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

  let gl_context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;
  shaders::do_webgl(gl_context)?;

  Ok(())
}
