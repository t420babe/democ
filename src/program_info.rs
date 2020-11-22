use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

const VERT_SOURCE: &str = r#"
    attribute vec4 a_vertex_position;
    attribute vec4 a_vertex_color;

    uniform mat4 u_model_view_matrix;
    uniform mat4 u_projection_matrix;

    varying lowp vec4 v_color;

    void main(void) {
      gl_Position = u_projection_matrix * u_model_view_matrix * a_vertex_position;
      v_color = a_vertex_color;
    }
  "#;

const FRAG_SOURCE: &str = r#"
    varying lowp vec4 v_color;

    void main() {
      gl_FragColor = v_color;
    }
"#;

pub struct ProgramInfo {
  pub program: WebGlProgram,
  pub attrib_locations: HashMap<String, i32>,
  pub uniform_locations: HashMap<String, Option<WebGlUniformLocation>>,
}

impl ProgramInfo {
  pub fn new(gl_context: &WebGl2RenderingContext) -> Result<Self, JsValue> {
    let shader_program = init_shader_program(&gl_context)?;
    let mut attrib_locations: HashMap<String, i32> = HashMap::new();

    attrib_locations.insert(
      "a_vertex_position".into(),
      gl_context.get_attrib_location(&shader_program, "a_vertex_position"),
    );

    attrib_locations.insert(
      "a_vertex_color".into(),
      gl_context.get_attrib_location(&shader_program, "a_vertex_color"),
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

    Ok(ProgramInfo { program: shader_program, attrib_locations, uniform_locations })
  }
}

fn init_shader_program(gl_context: &WebGl2RenderingContext) -> Result<WebGlProgram, String> {
  // Load shaders
  let vert_shader = load_shader(gl_context, VERT_SOURCE, WebGl2RenderingContext::VERTEX_SHADER)?;
  let frag_shader = load_shader(gl_context, FRAG_SOURCE, WebGl2RenderingContext::FRAGMENT_SHADER)?;

  // Create the shader program
  let shader_program = gl_context.create_program().ok_or_else(|| {
    let msg = "Unable to create shader object";
    console::log_1(&msg.into());
    msg.to_string()
  })?;
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
    Err(gl_context.get_program_info_log(&shader_program).unwrap_or_else(|| {
      let msg = "Unknown error creating program object";
      console::log_1(&msg.into());
      msg.to_string()
    }))
  }
}

fn load_shader(
  gl_context: &WebGl2RenderingContext,
  shader_source: &str,
  shader_type: u32,
) -> Result<WebGlShader, String> {
  let shader = gl_context.create_shader(shader_type).ok_or_else(|| {
    let msg = "Unable to create shader object";
    console::log_1(&msg.into());
    msg.to_string()
  })?;
  gl_context.shader_source(&shader, shader_source);
  gl_context.compile_shader(&shader);

  if gl_context
    .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(shader)
  } else {
    Err(gl_context.get_shader_info_log(&shader).unwrap_or_else(|| {
      let msg = "Unknown error creating shader";
      console::log_1(&msg.into());
      msg.to_string()
    }))
  }
}
