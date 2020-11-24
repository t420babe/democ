use std::convert::TryInto;

pub(crate) fn to_f32_8<T>(v: Vec<T>) -> [T; 8]
where
  T: Copy,
{
  let slice = v.as_slice();
  let array: [T; 8] = match slice.try_into() {
    Ok(ba) => ba,
    Err(_) => {
      panic!("Expected a `vec` of length `{}` but received a `vec` of length `{}`", 8, v.len())
    }
  };
  array
}

pub(crate) fn to_f32_16<T>(v: Vec<T>) -> [T; 16]
where
  T: Copy,
{
  let slice = v.as_slice();
  let array: [T; 16] = match slice.try_into() {
    Ok(ba) => ba,
    Err(_) => {
      panic!("Expected a `vec` of length `{}` but received a `vec` of length `{}`", 16, v.len())
    }
  };
  array
}

pub(crate) fn mat4_to_f32_16<T: 'static>(v: nalgebra_glm::TMat4<T>) -> [T; 16]
where
  T: Copy + PartialEq + std::fmt::Debug,
{
  let slice = v.as_slice();
  let array: [T; 16] = match slice.try_into() {
    Ok(ba) => ba,
    Err(_) => panic!("Expected"),
  };
  array
}
