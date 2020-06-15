#version 330 core

layout (location = 0) in vec3 _vertex;
layout (location = 1) in vec3 _norm;
layout (location = 2) in vec2 _uv;

// basic
uniform mat4 _projection;
uniform mat4 _view;
uniform mat4 _model;
uniform float _time;
uniform int _flip_uvs;
out vec2 _uv_coord;
out float _tm;
// TODO
out vec3 _normal;

// spritebatch
mat4 _sb_model;
uniform samplerBuffer _sb_instance_buffer;
out vec4 _sb_color;
out vec2 _sb_uv;

void ready_spritebatch() {
    int i_at = gl_InstanceID * 4;

    // uv_coords   [x1 y1 x2 y2]
    // vert_color  [ r  g  b  a]
    // trans_scale [ x  y sx sy]
    // last        [ r  _  _  _]
    vec4 uv_coords   = texelFetch(_sb_instance_buffer, i_at);
    _sb_color        = texelFetch(_sb_instance_buffer, i_at + 1);
    vec4 trans_scale = texelFetch(_sb_instance_buffer, i_at + 2);
    vec4 last        = texelFetch(_sb_instance_buffer, i_at + 3);

    // TODO flip uvs

    // get uv point from tex_coords
    switch (gl_VertexID) {
    case 0:
        _sb_uv = uv_coords.zw;
        break;
    case 1:
        _sb_uv = uv_coords.zy;
        break;
    case 2:
        _sb_uv = uv_coords.xw;
        break;
    case 3:
        _sb_uv = uv_coords.xy;
        break;
    }

    float rot = last.r;

    _sb_model = mat4(1.0);
    float rc = cos(rot);
    float rs = sin(rot);
    _sb_model[0][0] =  rc * trans_scale.z;
    _sb_model[0][1] =  rs * trans_scale.z;
    _sb_model[1][0] = -rs * trans_scale.w;
    _sb_model[1][1] =  rc * trans_scale.w;
    // _sb_model[3][0] = floor(trans_scale.x);
    // _sb_model[3][1] = floor(trans_scale.y);
    _sb_model[3][0] = trans_scale.x;
    _sb_model[3][1] = trans_scale.y;
}

@

vec4 effect() {
  return _projection * _view * _model * vec4(_vertex, 1.0);
}

@

void main() {
  _uv_coord = _flip_uvs != 0 ? vec2(_uv.x, 1 - _uv.y) : _uv;
  _tm = _time;
  gl_Position = effect();
}
