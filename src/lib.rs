use mat4;
use std::collections::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
  WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
};

pub struct ProgramInfo {
  program: WebGlProgram,
  attrib_locations: HashMap<String, i32>,
  uniform_locations: HashMap<String, Option<WebGlUniformLocation>>,
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();

  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

  let gl_context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

  let vert_source = r#"
      attribute vec4 a_vertex_position;

      uniform mat4 u_model_view_matrix;
      uniform mat4 u_projection_matrix;

      void main() {
        gl_Position = u_projection_matrix * u_model_view_matrix * a_vertex_position;
      }
  "#;

  let frag_source = r#"
    void main() {
      gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
  "#;
  let shader_program = init_shader_program(&gl_context, &vert_source, &frag_source)?;
  let mut attrib_locations: HashMap<String, i32> = HashMap::new();
  attrib_locations.insert(
    "a_vertex_position".into(),
    gl_context.get_attrib_location(&shader_program, "a_vertex_position"),
  );

  let mut uniform_locations: HashMap<String, Option<WebGlUniformLocation>> = HashMap::new();
  uniform_locations.insert(
    "u_model_view_matrix".into(),
    gl_context.get_uniform_location(&shader_program, "u_model_view_matrix"),
  );
  uniform_locations.insert(
    "u_projection_matrix".into(),
    gl_context.get_uniform_location(&shader_program, "u_projection_matrix"),
  );

  let program_info = ProgramInfo { program: shader_program, attrib_locations, uniform_locations };
  let vertices_buffer = init_buffers(&gl_context)?;
  draw_scene(&gl_context, &program_info, &vertices_buffer)?;

  Ok(())
}

pub fn draw_scene(
  gl_context: &WebGl2RenderingContext,
  program_info: &ProgramInfo,
  vertices_buffer: &WebGlBuffer,
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
    .ok_or("Failed to get canvas on draw")?
    .dyn_into::<web_sys::HtmlCanvasElement>()?;
  let aspect = (canvas.client_width() / canvas.client_height()) as f32;
  let z_near = 0.1;
  let z_far = 100.0;
  let mut projection_matrix: [f32; 16] = mat4::new_identity();
  mat4::perspective(&mut projection_matrix, &field_of_view, &aspect, &z_near, &z_far);

  // Set the drawing position to the "identity", which is the center of the scene
  let mut model_view_matrix: [f32; 16] = mat4::new_identity();
  let mut model_view_matrix_clone: [f32; 16] = model_view_matrix.clone();

  // Move the drawing position to where we want to start drawing the square
  // (destination matrix, matrix to translate, amount to translate)
  mat4::translate(&mut model_view_matrix, &model_view_matrix_clone, &[-0.0, 0.0, -6.0]);

  // Tell WebGl to pull out the positions from the vertices buffer into the `a_vertex_position` attribute
  let num_components = 2; // Pull out 2 values per iteration
  let buffer_type = WebGl2RenderingContext::FLOAT; // The data buffer is a 32bit float
  let normalize = false; // Do not normalize
  let stride = 0; // How many bytes to get from one set of values to the next, 0 = use `buffer_type` and `num_components`
  let offset = 0; // How many bytes inside the buffer to start from

  gl_context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertices_buffer));
  let a_vertex_position = (*program_info
    .attrib_locations
    .get(&"a_vertex_position".to_string())
    .ok_or("Failed to get `a_vertex_position` attribute")?) as u32;

  gl_context.vertex_attrib_pointer_with_i32(
    a_vertex_position,
    num_components,
    buffer_type,
    normalize,
    stride,
    offset,
  );

  gl_context.enable_vertex_attrib_array(a_vertex_position);

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
  gl_context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, offset, vertex_count);

  Ok(())
}

pub fn init_buffers(gl_context: &WebGl2RenderingContext) -> Result<WebGlBuffer, String> {
  // Create a buffer for the square's positions
  let vertices_buffer = gl_context.create_buffer().ok_or("Failed to create position buffer")?;

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

pub fn init_shader_program(
  gl_context: &WebGl2RenderingContext,
  vert_sourrce: &str,
  frag_source: &str,
) -> Result<WebGlProgram, String> {
  // Load shaders
  let vert_shader = load_shader(gl_context, vert_sourrce, WebGl2RenderingContext::VERTEX_SHADER)?;
  let frag_shader = load_shader(gl_context, frag_source, WebGl2RenderingContext::FRAGMENT_SHADER)?;

  // Create the shader program
  let shader_program =
    gl_context.create_program().ok_or_else(|| String::from("Unable to create shader object"))?;
  gl_context.attach_shader(&shader_program, &vert_shader);
  gl_context.attach_shader(&shader_program, &frag_shader);
  gl_context.link_program(&shader_program);

  if gl_context
    .get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(shader_program)
  } else {
    Err(
      gl_context
        .get_program_info_log(&shader_program)
        .unwrap_or_else(|| String::from("Unknown error creating program object")),
    )
  }
}

pub fn load_shader(
  gl_context: &WebGl2RenderingContext,
  shader_source: &str,
  shader_type: u32,
) -> Result<WebGlShader, String> {
  let shader = gl_context
    .create_shader(shader_type)
    .ok_or_else(|| String::from("Unable to create shader object"))?;
  gl_context.shader_source(&shader, shader_source);
  gl_context.compile_shader(&shader);

  if gl_context
    .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(shader)
  } else {
    Err(
      gl_context
        .get_shader_info_log(&shader)
        .unwrap_or_else(|| String::from("Unknown error creating shader")),
    )
  }
}
