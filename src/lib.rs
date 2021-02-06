use mat4;
use std::{cell::RefCell, rc::Rc, collections::HashMap};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, console, Window, WebGl2RenderingContext, WebGlBuffer};

mod buffer_attrib;
mod buffers;
mod program_info;
mod utils;
use crate::{buffer_attrib::BufferAttrib, program_info::ProgramInfo};

fn window() -> web_sys::Window {
  web_sys::window().expect("No global `window` exists")
}

fn document() -> web_sys::Document {
  window().document().expect("Should have a `document` on `window`")
}

fn body() -> web_sys::HtmlElement {
  document().body().expect("Should have a `body` on `document`")
}


#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let audio_context = AudioContext::new()?;
  let stream = get_stream(&window, &audio_context).await?;

  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

  let gl_context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

  let program_info = ProgramInfo::new(&gl_context)?;

  let buffers = buffers::make_buffers(&gl_context)?;

  draw_scene(&gl_context, &program_info, &buffers)?;

  Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
  window().request_animation_frame(f.as_ref().unchecked_ref())
    .expect("Should register `requestAnimationFrame` OK");

}

async fn get_stream(window: &Window, audio_context: &AudioContext) -> Result<(), JsValue> {
  console::log_1(&1.into());
  let node = audio_context.create_analyser()?;

  let nav = window.navigator();
  let media_devices = nav.media_devices()?;

  // May need to use MediaTrackConstraints here to invoke noise_suppression and echo_cancellation
  let mut media_constraints = web_sys::MediaStreamConstraints::new();
  &media_constraints.audio(&JsValue::TRUE);
  &media_constraints.video(&JsValue::FALSE);
  let stream_value = JsFuture::from(media_devices.get_user_media_with_constraints(&media_constraints)?).await?;

  assert!(stream_value.is_instance_of::<web_sys::MediaStream>());
  let stream: web_sys::MediaStream = stream_value.dyn_into().unwrap();
  let audio_node = &audio_context.create_media_stream_source(&stream)?;

  let f = Rc::new(RefCell::new(None));
  let g = f.clone();
  let k_max_freq: f32 = 20000.0;
  let mut buffer_size = (k_max_freq / &audio_context.sample_rate() * (node.fft_size() as f32 / 2.0)).floor() as usize;
  let mut buffer_vec = vec![0; buffer_size];
  let mut buffer: &mut [u8] = &mut buffer_vec;
  &node.get_byte_frequency_data(buffer);
  // let mut buffer_array = js_sys::Uint8Array::view(buffer);
  // console::log(&buffer_array.into());
  let buffer_last: u32 = *buffer.last().unwrap() as u32;
  console::log_1(&buffer_last.into());
  let mut i: u32 = 0;
  *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
    if i > 300 {
      body().set_text_content(Some("All done"));
      console::log_1(&i.into());
      let _ = f.borrow_mut().take();
      return;
    }
    // &node.get_byte_frequency_data(&mut [buffer_size]);
    // &node.get_byte_frequency_data(&mut [buffer_size]);
    // console::log_1(&buffer_size.into());
    i += 1;
    let text = format!("requestAnimationFrame has been called {} times", i);
    body().set_text_content(Some(&text));

    request_animation_frame(f.borrow().as_ref().unwrap());
  }) as Box<dyn FnMut()>));

  request_animation_frame(g.borrow().as_ref().unwrap());


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

  // Projection and model view matrices
  let projection_matrix = create_perspective_matrix(&gl_context)?;
  let model_view_matrix = create_model_view_matrix();

  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  let a_vertex_position =
    (*program_info.attrib_locations.get(&"a_vertex_position".to_string()).ok_or({
      let msg = "Failed to get `a_vertex_position` attribute";
      console::log_1(&msg.into());
      msg
    })?) as u32;

  let a_vertex_position_buffer_attrib = BufferAttrib {
    name: "vertices".into(),
    buffer: buffers.get(&"vertices".to_string()).ok_or({
      let msg = "Failed to get `a_vertex_position` attribute";
      console::log_1(&msg.into());
      msg
    })?,
    target: WebGl2RenderingContext::ARRAY_BUFFER,
    num_components: 2,
    buffer_type: WebGl2RenderingContext::FLOAT,
    normalize: false,
    stride: 0,
    offset: 0,
  };
  buffer_attrib::bind_buffer_to_attrib(
    &gl_context,
    &a_vertex_position_buffer_attrib,
    a_vertex_position,
  )?;

  let a_vertex_color =
    (*program_info.attrib_locations.get(&"a_vertex_color".to_string()).ok_or({
      let msg = "Failed to get `a_vertex_color` attribute";
      console::log_1(&msg.into());
      msg
    })?) as u32;
  let a_vertex_color_buffer_attrib = BufferAttrib {
    name: "colors".into(),
    buffer: buffers.get(&"colors".to_string()).ok_or({
      let msg = "Failed to get `a_vertex_color` attribute";
      console::log_1(&msg.into());
      msg
    })?,
    target: WebGl2RenderingContext::ARRAY_BUFFER,
    num_components: 4,
    buffer_type: WebGl2RenderingContext::FLOAT,
    normalize: false,
    stride: 0,
    offset: 0,
  };
  buffer_attrib::bind_buffer_to_attrib(&gl_context, &a_vertex_color_buffer_attrib, a_vertex_color)?;

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

fn create_perspective_matrix(gl_context: &WebGl2RenderingContext) -> Result<[f32; 16], JsValue> {
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
  Ok(*mat4::perspective(&mut projection_matrix, &field_of_view, &aspect, &z_near, &z_far))
}

fn create_model_view_matrix() -> [f32; 16] {
  // Set the drawing position to the "identity", which is the center of the scene
  let mut model_view_matrix: [f32; 16] = mat4::new_identity();
  let model_view_matrix_clone: [f32; 16] = model_view_matrix.clone();

  // Move the drawing position to where we want to start drawing the square
  // (destination matrix, matrix to translate, amount to translate)
  *mat4::translate(&mut model_view_matrix, &model_view_matrix_clone, &[-0.0, 0.0, -6.0])
}
