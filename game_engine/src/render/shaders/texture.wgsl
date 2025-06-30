// Vertex shader
struct InstanceInput {
  @location(5) model_matrix_0: vec4<f32>,
  @location(6) model_matrix_1: vec4<f32>,
  @location(7) model_matrix_2: vec4<f32>,
  @location(8) model_matrix_3: vec4<f32>,
  @location(9) texture_index: u32,
  @location(10) normal_matrix_0: vec3<f32>,
  @location(11) normal_matrix_1: vec3<f32>,
  @location(12) normal_matrix_2: vec3<f32>,
};


struct CameraUniform {
  view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct LightUniform {
  position: vec3<f32>,
  color: vec3<f32>,
};
@group(2) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) texture_coords: vec2<f32>,
  @location(2) normal: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) texture_coords: vec2<f32>,
  @location(1) texture_index: u32,
  @location(2) world_normal: vec3<f32>,
  @location(3) world_position: vec3<f32>,
};

@vertex
fn vs_main(
  model: VertexInput,
  instance: InstanceInput,
) -> VertexOutput {
  let model_matrix = mat4x4<f32>(
    instance.model_matrix_0,
    instance.model_matrix_1,
    instance.model_matrix_2,
    instance.model_matrix_3,
  );
  let normal_matrix = mat3x3<f32>(
    instance.normal_matrix_0,
    instance.normal_matrix_1,
    instance.normal_matrix_2,
  );

  var out: VertexOutput;

  out.texture_coords = model.texture_coords;
  out.texture_index = instance.texture_index;

  var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
  out.world_position = world_position.xyz;
  out.clip_position = camera.view_proj * world_position;

  out.world_normal = normal_matrix * model.normal;

  return out;
}

// Fragment shader
@group(0) @binding(0)
var my_texture: texture_2d_array<f32>;
@group(0) @binding(1)
var my_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let ambient_strength = 0.1; // Strength of the light

  let object_color = textureSample(my_texture, my_sampler, in.texture_coords, in.texture_index);

  // Ambient - Overall light level
  let ambient_color = light.color * ambient_strength;

  // Diffuse - How the light angle hits the block face
  let light_direction = normalize(light.position - in.world_position);
  let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0); 
  let diffuse_color = light.color * diffuse_strength;

  // TODO: Specular - Light reflections

  let blended_color = (ambient_color + diffuse_color) * object_color.xyz;

  return vec4<f32>(blended_color, object_color.a);
}
