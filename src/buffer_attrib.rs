use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

pub struct BufferAttrib<'a> {
  pub name: String,
  pub buffer: &'a WebGlBuffer,
  pub target: u32,
  pub num_components: i32,
  pub buffer_type: u32,
  pub normalize: bool,
  pub stride: i32,
  pub offset: i32,
}

pub fn bind_buffer_to_attrib(
  gl_context: &WebGl2RenderingContext,
  buffer_attrib: &BufferAttrib,
  attribute: u32,
) -> Result<(), JsValue> {
  gl_context.bind_buffer(buffer_attrib.target, Some(buffer_attrib.buffer));

  gl_context.vertex_attrib_pointer_with_i32(
    attribute,
    buffer_attrib.num_components,
    buffer_attrib.buffer_type,
    buffer_attrib.normalize,
    buffer_attrib.stride,
    buffer_attrib.offset,
  );

  gl_context.enable_vertex_attrib_array(attribute);

  Ok(())
}
