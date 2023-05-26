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

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out : VertexOutput;

    out.clip = model * view_proj * vec4<f32>(in.position, 1.0);
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

//    out.color = textureSample(diffuse_texture, diffuse_sampler, in.texcoord);
    out.albedo_attachment = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    out.normal_attachment = vec4<f32>(in.normal, 1.0);

    return out;
}
