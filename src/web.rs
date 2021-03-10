use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
  console, AudioContext, HtmlCanvasElement, HtmlMediaElement, WebGl2RenderingContext,
  WebGlBuffer,
  AnalyserNode,
};
use crate::{
  shaders, buffer_attrib, buffer_attrib::BufferAttrib, buffers, program_info::ProgramInfo, utils::*,
};

fn draw_loop(node: &AnalyserNode, mut buffer: Vec<u8>, 
  gl_context: &WebGl2RenderingContext,
  program_info: ProgramInfo,
  buffers: HashMap<String, WebGlBuffer>,
  time: f32,
  ) -> Result<(), JsValue> {
  // WebAudio
  &node.get_byte_frequency_data(&mut buffer);
  // web_sys::console::log(&super::vec_to_js_array(buffer));

  // WebGL
  gl_context.clear_color(1.0, 0.5, 0.5, 1.0);
  // gl_context.clear_depth(0.0);
  gl_context.enable(WebGl2RenderingContext::DEPTH_TEST);
  gl_context.depth_func(WebGl2RenderingContext::LEQUAL); // Near objects obscure far ones

  // Clear the canvas before drawing to it
  gl_context
    .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

  // Projection and model view matrices
  let projection_matrix = shaders::create_perspective_matrix(&gl_context)?;
  let model_view_matrix = shaders::create_model_view_matrix(time);

  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  let a_vertex_position = (*program_info
    .attrib_locations
    .get(&"a_vertex_position".to_string())
    .ok_or("Failed to get `a_vertex_position` attribute")?) as u32;

  let a_vertex_position_buffer_attrib = BufferAttrib {
    name: "vertices".into(),
    buffer: buffers
      .get(&"vertices".to_string())
      .ok_or("Failed to get `a_vertex_position` attribute")?,
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

  let a_vertex_color = (*program_info
    .attrib_locations
    .get(&"a_vertex_color".to_string())
    .ok_or("Failed to get `a_vertex_color` attribute")?) as u32;
  let a_vertex_color_buffer_attrib = BufferAttrib {
    name: "colors".into(),
    buffer: buffers.get(&"colors".to_string()).ok_or("Failed to get `a_vertex_color` attribute")?,
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

  gl_context
    .uniform1f(program_info.uniform_locations.get(&"u_time".to_string()).unwrap().as_ref(), time);

  gl_context
    .uniform1f(program_info.uniform_locations.get(&"u_one".to_string()).unwrap().as_ref(), 1.0f32);

  let vertex_count = 4;
  let offset = 0; // How many bytes inside the buffer to start from
  gl_context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, offset, vertex_count);
  Ok(())
}

pub(crate) async fn run(audio_context: AudioContext, gl_context: WebGl2RenderingContext) -> Result<(), JsValue> {
  // Setup WebAudio
  let node = audio_context.create_analyser()?;
  let navigator = &super::window().navigator();
  let mut media_stream_constraints = web_sys::MediaStreamConstraints::new();
  &media_stream_constraints.audio(&JsValue::TRUE);
  &media_stream_constraints.video(&JsValue::FALSE);
  let stream_promise = navigator.media_devices()?.get_user_media_with_constraints(&media_stream_constraints)?;
  let stream: web_sys::MediaStream = JsFuture::from(stream_promise).await?.dyn_into()?;

  // Buffer to hold fft data
  let kMaxFrequency = 20000.0f32;
  let sample_rate = audio_context.sample_rate() as f32;
  let fft_size = (node.fft_size() / 2) as f32;   // 1024
  let audio_buffer_size = (kMaxFrequency / sample_rate* fft_size) as usize;
  let mut audio_buffer = vec![0; audio_buffer_size];

  let audio_node = &audio_context.create_media_stream_source(&stream)?;
  audio_node.connect_with_audio_node(&node)?;

  // Setup WebGl shaders 
  let program_info = ProgramInfo::new(&gl_context)?;
  let buffers = buffers::make_buffers(&gl_context)?;

  // Draw scene every 0.01 seconds
  let ref_count = Rc::new(RefCell::new(None));
  let ref_count_clone = ref_count.clone();

  *ref_count_clone.borrow_mut() = Some(Closure::wrap(Box::new(move |t| {
    let audio_buf = audio_buffer.clone();
    draw_loop(&node, audio_buf, &gl_context.clone(), program_info.clone(), buffers.clone(), t * 0.001f32);

    request_animation_frame(ref_count.borrow().as_ref().unwrap());
  }) as Box<dyn FnMut(f32)>));

  request_animation_frame(ref_count_clone.borrow().as_ref().unwrap());
  Ok(())
}

// /// Audio draw loop
// fn draw_loop(node: &AnalyserNode, mut buffer: Vec<u8>) -> Result<(), JsValue> {
//   &node.get_byte_frequency_data(&mut buffer);
//   web_sys::console::log(&vec_to_js_array(buffer));
//   Ok(())
// }

fn request_animation_frame(f: &Closure<dyn FnMut(f32)>) {
  super::window()
    .request_animation_frame(f.as_ref().unchecked_ref())
    .expect("Error. Did not register `RequestAnimationFrame`");
}

async fn setup_webaudio(audio_context: AudioContext) -> Result<(), JsValue> {

  Ok(())
}
