struct VertexInput {
    @location(0) position : vec2<f32>,
    @location(1) texcoord : vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip     : vec4<f32>,
    @location(0)       texcoord : vec2<f32>,
}

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out : VertexOutput;

    out.clip = vec4<f32>(in.position * 2.0 - 1.0, 0.0, 1.0);
    out.texcoord = in.texcoord;

    return out;
}


@group(0) @binding(0) var albedo_texture : texture_2d<f32>;
@group(0) @binding(1) var normal_texture : texture_2d<f32>;
@group(0) @binding(2) var depth_texture  : texture_2d<f32>;
@group(0) @binding(3) var target_sampler : sampler;

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(albedo_texture, target_sampler, in.texcoord);
}
