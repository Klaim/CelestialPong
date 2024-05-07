use macroquad::prelude::*;
use macroquad::color::colors;
use macroquad::ui::{hash, root_ui, widgets, Skin};

pub fn get_circe_arrow(width : u32, height : u32, color: Color) -> Texture2D {
    let render_target = render_target(width, height);
    render_target.texture.set_filter(FilterMode::Nearest);

    let material = load_material(
        ShaderSource::Glsl {
            vertex: &DEFAULT_VERTEX_SHADER,
            fragment: &ARROW_FRAGMENT_SHADER,
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

pub fn get_anti_clockwise_skin(width: f32, height: f32) -> Skin {

    let texture = get_circe_arrow(width as u32, height as u32, colors::WHITE).get_texture_data();
    let hovered = get_circe_arrow(width as u32, height as u32, colors::LIGHTGRAY).get_texture_data();
    let clicked = get_circe_arrow(width as u32, height as u32, colors::BEIGE).get_texture_data();

    let arrow_button_style = root_ui().style_builder()
        .background(texture)
        .background_hovered(hovered)
        .background_clicked(clicked)
        .margin(RectOffset{top:height / 2.,left:width / 2.,bottom:height / 2.,right:width / 2.})
        .font_size(0)
        .build();

    Skin {
        button_style: arrow_button_style,
        ..root_ui().default_skin()
    }
}

const ARROW_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;
varying vec4 color;

const float TAU = 6.28318;
const highp float NOISE_GRANULARITY = 4./255.;

highp float random(highp vec2 coords) {
   return fract(sin(dot(coords.xy, vec2(12.9898,78.233))) * 43758.5453);
}

 // From https://iquilezles.org/articles/distfunctions2d/  
float udSegment( in vec2 p, in vec2 a, in vec2 b )
{
    vec2 ba = b-a;
    vec2 pa = p-a;
    float h =clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
    return length(pa-h*ba);
}

float sdf(vec2 uv, float radius, float arrowRadius)
{
    float headDelta = radius * 4.;
    vec2 arrowPoint = vec2(arrowRadius, 0.);
    
    // eyeballing a delta for the arrow head that looks okay
    float head = min(
        udSegment(uv, arrowPoint, arrowPoint - vec2(headDelta, -headDelta)),
        udSegment(uv, arrowPoint, arrowPoint - vec2(-headDelta * .8, -headDelta * 1.2))) 
        - radius;
    
    // Going to polar coordinate
    float angle = (atan(uv.y, uv.x));
    float len = length(uv);
    // distoring space
    angle = angle - clamp(angle, 0., TAU / 4.);
    // Going back to cartesian
    uv = vec2(cos(angle), sin(angle)) * len;
    // a point along the 0 y axis will now be streched into an arc
    float body = length(uv - vec2(arrowRadius, 0.)) - radius;
    
    return min(body, head);
}

void main() {
    float d = sdf(uv, .05, .75);
    d = smoothstep(-.01, .01, -d);
    //d += mix(-NOISE_GRANULARITY, NOISE_GRANULARITY, random(uv));
    gl_FragColor = color * (d + .1);
}
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
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
