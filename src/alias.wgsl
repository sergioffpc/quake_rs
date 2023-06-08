@group(0) @binding(0) var<uniform> view_proj : mat4x4f;
@group(1) @binding(0) var<uniform> model     : mat4x4f;

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) texcoord : vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip     : vec4<f32>,
    @location(0)       normal   : vec3<f32>,
    @location(1)       texcoord : vec2<f32>,
}

/* The Quake coordinate system defines X as the longitudinal axis, Y as the
 * lateral axis, and Z as the vertical axis.  */
fn from_quake_coords(coords: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(-coords.y, coords.z, -coords.x);
}

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out : VertexOutput;

    out.clip = model * view_proj * vec4<f32>(from_quake_coords(in.position), 1.0);
    out.normal = in.normal;
    out.texcoord = in.texcoord;

    return out;
}


@group(2) @binding(0) var diffuse_texture : texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler : sampler;

struct FragmentOutput {
    @location(0) albedo_attachment : vec4<f32>,
    @location(1) normal_attachment : vec4<f32>,
}

@fragment fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    out.albedo_attachment = textureSample(diffuse_texture, diffuse_sampler, in.texcoord);
    out.normal_attachment = vec4<f32>(in.normal, 1.0);

    return out;
}
