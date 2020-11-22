use crate::utils::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, WebGl2RenderingContext, WebGlBuffer};

pub fn make_buffers(
  gl_context: &WebGl2RenderingContext,
) -> Result<HashMap<String, WebGlBuffer>, JsValue> {
  let mut buffers: HashMap<String, WebGlBuffer> = HashMap::new();

  let target = WebGl2RenderingContext::ARRAY_BUFFER;
  let usage = WebGl2RenderingContext::STATIC_DRAW;

  // Create an array of position vertices for the square
  let position_vertices = vec![-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0];

  // Create an array of color vertices for the square
  let color_vertices = vec![
    1.0, 1.0, 1.0, 1.0, // white
    1.0, 0.0, 0.0, 1.0, // red
    0.0, 1.0, 0.0, 1.0, // green
    0.0, 0.0, 1.0, 1.0, // blue
  ];

  let vertices_buffer = init_buffer(&gl_context, position_vertices, target, usage)?;
  buffers.insert("vertices".into(), vertices_buffer);

  let colors_buffer = init_buffer(&gl_context, color_vertices, target, usage)?;
  buffers.insert("colors".into(), colors_buffer);

  Ok(buffers)
}

fn init_buffer(
  gl_context: &WebGl2RenderingContext,
  vertices: Vec<f32>,
  target: u32,
  usage: u32,
) -> Result<WebGlBuffer, String> {
  // Create a buffer for the square's positions
  let buffer = gl_context.create_buffer().ok_or({
    let msg = "Failed to create position buffer";
    console::log_1(&msg.into());
    msg
  })?;

  // Select the `vertices_buffer` as the on to apply buffer operations from here on out
  gl_context.bind_buffer(target, Some(&buffer));

  if vertices.len() == 8 {
    let vertices_tmp: [f32; 8] = to_f32_8(vertices);
    // Pass the list of vertices into WebGl to build the shape
    unsafe {
      let vertices_array = js_sys::Float32Array::view(&vertices_tmp);
      gl_context.buffer_data_with_array_buffer_view(target, &vertices_array, usage);
    }
  } else if vertices.len() == 16 {
    let vertices_tmp: [f32; 16] = to_f32_16(vertices);
    // Pass the list of vertices into WebGl to build the shape
    unsafe {
      let vertices_array = js_sys::Float32Array::view(&vertices_tmp);
      gl_context.buffer_data_with_array_buffer_view(target, &vertices_array, usage);
    }
  } else {
    panic!(
      "Expected a `vec` of length `8` or `16` but received a `vec` of length `{}`",
      vertices.len()
    );
  }

  Ok(buffer)
}
