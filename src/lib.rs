use mat4;
use std::collections::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, WebGl2RenderingContext, WebGlBuffer};

pub mod buffers;
pub mod program_info;
pub use program_info::ProgramInfo;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();

  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

  let gl_context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

  let program_info = ProgramInfo::new(&gl_context)?;

  let buffers = buffers::make_buffers(&gl_context)?;

  draw_scene(&gl_context, &program_info, &buffers)?;

  Ok(())
}

fn bind_buffer_to_a_vertex_position_attrib(
  gl_context: &WebGl2RenderingContext,
  program_info: &ProgramInfo,
  buffers: &HashMap<String, WebGlBuffer>,
) -> Result<(), JsValue> {
  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  let num_components = 2; // Pull out 2 values per iteration
  let buffer_type = WebGl2RenderingContext::FLOAT; // The data buffer is a 32bit float
  let normalize = false; // Do not normalize
  let stride = 0; // How many bytes to get from one set of values to the next, 0 = use `buffer_type` and `num_components`
  let offset = 0; // How many bytes inside the buffer to start from

  gl_context
    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, buffers.get(&"vertices".to_string()));

  let a_vertex_position =
    (*program_info.attrib_locations.get(&"a_vertex_position".to_string()).ok_or({
      let msg = "Failed to get `a_vertex_position` attribute";
      console::log_1(&msg.into());
      msg
    })?) as u32;

  gl_context.vertex_attrib_pointer_with_i32(
    a_vertex_position,
    num_components,
    buffer_type,
    normalize,
    stride,
    offset,
  );

  gl_context.enable_vertex_attrib_array(a_vertex_position);
  Ok(())
}

fn bind_buffer_to_a_vertex_color_attrib(
  gl_context: &WebGl2RenderingContext,
  program_info: &ProgramInfo,
  buffers: &HashMap<String, WebGlBuffer>,
) -> Result<(), JsValue> {
  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  let num_components = 4; // Pull out 2 values per iteration
  let buffer_type = WebGl2RenderingContext::FLOAT; // The data buffer is a 32bit float
  let normalize = false; // Do not normalize
  let stride = 0; // How many bytes to get from one set of values to the next, 0 = use `buffer_type` and `num_components`
  let offset = 0; // How many bytes inside the buffer to start from

  gl_context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, buffers.get(&"colors".to_string()));
  let a_vertex_color =
    (*program_info.attrib_locations.get(&"a_vertex_color".to_string()).ok_or({
      let msg = "Failed to get `a_vertex_color` attribute";
      console::log_1(&msg.into());
      msg
    })?) as u32;

  gl_context.vertex_attrib_pointer_with_i32(
    a_vertex_color,
    num_components,
    buffer_type,
    normalize,
    stride,
    offset,
  );

  gl_context.enable_vertex_attrib_array(a_vertex_color);
  Ok(())
}

pub fn draw_scene(
  gl_context: &WebGl2RenderingContext,
  program_info: &ProgramInfo,
  buffers: &HashMap<String, WebGlBuffer>,
) -> Result<(), JsValue> {
  gl_context.clear_color(1.0, 0.5, 0.5, 1.0);
  // gl_context.clear_depth(0.0);
  gl_context.enable(WebGl2RenderingContext::DEPTH_TEST);
  gl_context.depth_func(WebGl2RenderingContext::LEQUAL); // Near objects obscure far ones

  // Clear the canvas before drawing to it
  gl_context
    .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

  // Create a perspective matrix, a special matrix that is used to simulate the distortion of perspective in a camera.
  // Our field of view is 45 degrees, which a width/height ratio that matches the display size of the canvas and we
  // only want to see objects between 0.1 and 100.0 units away from the camera
  let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
  let canvas: web_sys::HtmlCanvasElement = gl_context
    .canvas()
    .ok_or({
      let msg = "Failed to get canvas on draw";
      console::log_1(&msg.into());
      msg
    })?
    .dyn_into::<web_sys::HtmlCanvasElement>()?;
  let aspect = (canvas.client_width() / canvas.client_height()) as f32;
  let z_near = 0.1;
  let z_far = 100.0;
  let mut projection_matrix: [f32; 16] = mat4::new_identity();
  mat4::perspective(&mut projection_matrix, &field_of_view, &aspect, &z_near, &z_far);

  // Set the drawing position to the "identity", which is the center of the scene
  let mut model_view_matrix: [f32; 16] = mat4::new_identity();
  let model_view_matrix_clone: [f32; 16] = model_view_matrix.clone();

  // Move the drawing position to where we want to start drawing the square
  // (destination matrix, matrix to translate, amount to translate)
  mat4::translate(&mut model_view_matrix, &model_view_matrix_clone, &[-0.0, 0.0, -6.0]);

  /* BEGIN */
  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  bind_buffer_to_a_vertex_position_attrib(&gl_context, &program_info, &buffers)?;
  bind_buffer_to_a_vertex_color_attrib(&gl_context, &program_info, &buffers)?;
  /* END */

  // Tell WebGl to use our program when drawing
  gl_context.use_program(Some(&program_info.program));

  let projection_matrix = &projection_matrix[0..];
  gl_context.uniform_matrix4fv_with_f32_array(
    program_info.uniform_locations.get(&"u_projection_matrix".to_string()).unwrap().as_ref(),
    false,
    projection_matrix,
  );

  let model_view_matrix = &model_view_matrix[0..];
  gl_context.uniform_matrix4fv_with_f32_array(
    program_info.uniform_locations.get(&"u_model_view_matrix".to_string()).unwrap().as_ref(),
    false,
    model_view_matrix,
  );

  let vertex_count = 4;
  let offset = 0; // How many bytes inside the buffer to start from
  gl_context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, offset, vertex_count);

  Ok(())
}
