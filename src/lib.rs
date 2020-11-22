use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();

  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

  let gl_context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

  gl_context.clear_color(1.0, 0.5, 0.5, 1.0);
  gl_context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

  let vert_source = r#"
      attribute vec4 aVertexPosition;

      uniform mat4 uModelViewMatrix;
      uniform mat4 uProjectionMatrix;

      void main() {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
      }
  "#;

  let frag_source = r#"
    void main() {
      gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
  "#;
  let shader_program = init_shader_program(&gl_context, &vert_source, &frag_source)?;

  let a_vertex_position = gl_context.get_attrib_location(&shader_program, "aVertexPosition");
  let u_model_view_matrix = gl_context.get_attrib_location(&shader_program, "uModelViewMatrix");
  let u_projection_matrix = gl_context.get_attrib_location(&shader_program, "uProjectionMatrix");

  Ok(())
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
