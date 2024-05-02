use macroquad::prelude::*;

pub fn get_radial_gradient_texture(width: u32, height: u32, color: Color) -> Texture2D {
    let render_target = render_target(width, height);
    render_target.texture.set_filter(FilterMode::Nearest);

    let material = load_material(
        ShaderSource::Glsl {
            vertex: &RADIAL_VERTEX_SHADER,
            fragment: &RADIAL_FRAGMENT_SHADER,
        },
        Default::default(),
    )
    .unwrap();

    set_camera(&Camera2D {
        zoom: vec2(1., 1.),
        target: vec2(0., 0.),
        render_target: Some(render_target.clone()),
        ..Default::default()
    });
    {
        gl_use_material(&material);
        {
            draw_rectangle(-1., -1., 2., 2., color);
        }
        gl_use_default_material();
    }

    set_default_camera();
    return render_target.texture;
}

const RADIAL_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;
varying vec4 color;

const highp float NOISE_GRANULARITY = 4./255.;

highp float random(highp vec2 coords) {
   return fract(sin(dot(coords.xy, vec2(12.9898,78.233))) * 43758.5453);
}

void main() {
    float d = length(uv);
    d += mix(-NOISE_GRANULARITY, NOISE_GRANULARITY, random(uv));
    gl_FragColor = color * 1. - d;
}
";

const RADIAL_VERTEX_SHADER: &'static str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = (texcoord - vec2(.5)) * 2.;
    color = color0 / 255.;
}
";
