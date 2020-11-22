use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, WebGl2RenderingContext, WebGlBuffer};

pub fn make_buffers(
  gl_context: &WebGl2RenderingContext,
) -> Result<HashMap<String, WebGlBuffer>, JsValue> {
  let mut buffers: HashMap<String, WebGlBuffer> = HashMap::new();
  let vertices_buffer = init_buffers(&gl_context)?;
  buffers.insert("vertices".into(), vertices_buffer);
  let colors_buffer = init_color_buffer(&gl_context)?;
  buffers.insert("colors".into(), colors_buffer);

  Ok(buffers)
}

fn init_buffers(gl_context: &WebGl2RenderingContext) -> Result<WebGlBuffer, String> {
  // Create a buffer for the square's positions
  let vertices_buffer = gl_context.create_buffer().ok_or({
    let msg = "Failed to create position buffer";
    console::log_1(&msg.into());
    msg
  })?;

  // Select the `vertices_buffer` as the on to apply buffer operations from here on out
  gl_context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertices_buffer));

  // Create an array of vertices for the square
  let vertices: [f32; 8] = [-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0];

  // Pass the list of vertices into WebGl to build the shape

  unsafe {
    let vertices_array = js_sys::Float32Array::view(&vertices);
    gl_context.buffer_data_with_array_buffer_view(
      WebGl2RenderingContext::ARRAY_BUFFER,
      &vertices_array,
      WebGl2RenderingContext::STATIC_DRAW,
    );
  }

  Ok(vertices_buffer)
}

fn init_color_buffer(gl_context: &WebGl2RenderingContext) -> Result<WebGlBuffer, String> {
  // Create a buffer for the square's color
  let colors_buffer = gl_context.create_buffer().ok_or({
    let msg = "Failed to create position buffer";
    console::log_1(&msg.into());
    msg
  })?;

  // Select the `buffer` as the on to apply buffer operations from here on out
  gl_context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&colors_buffer));

  // Create an array of vertices for the square
  let colors: [f32; 16] = [
    1.0, 1.0, 1.0, 1.0, // white
    1.0, 0.0, 0.0, 1.0, // red
    0.0, 1.0, 0.0, 1.0, // green
    0.0, 0.0, 1.0, 1.0, // blue
  ];

  unsafe {
    let colors_array = js_sys::Float32Array::view(&colors);
    gl_context.buffer_data_with_array_buffer_view(
      WebGl2RenderingContext::ARRAY_BUFFER,
      &colors_array,
      WebGl2RenderingContext::STATIC_DRAW,
    );
  }

  Ok(colors_buffer)
}
