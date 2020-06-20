#version 330 core

layout (location = 0) in vec3 _ext_vertex;
layout (location = 1) in vec3 _ext_normal;
layout (location = 2) in vec2 _ext_uv;

// basic
uniform mat4 _projection;
uniform mat4 _view;
uniform mat4 _model;
uniform float _time;
uniform int _flip_uvs;

out vec2 _uv_coord;
out float _tm;
out vec3 _normal;

// spritebatch
mat4 _sb_model;
uniform samplerBuffer _sb_instance_buffer;
out vec4 _sb_color;
out vec2 _sb_uv;

mat4 mat4_from_transform2d(float x, float y, float sx, float sy, float r) {
    mat4 ret = mat4(1.0);
    float rc = cos(r);
    float rs = sin(r);
    ret[0][0] =  rc * sx;
    ret[0][1] =  rs * sx;
    ret[1][0] = -rs * sy;
    ret[1][1] =  rc * sy;
    ret[3][0] = x;
    ret[3][1] = y;
    return ret;
}

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

    _sb_model = mat4_from_transform2d(trans_scale.x, trans_scale.y, trans_scale.z, trans_scale.w, last.x);
}

@

vec4 effect() {
  return _projection * _view * _model * vec4(_ext_vertex, 1.0);
}

@

void main() {
  _uv_coord = _flip_uvs != 0 ? vec2(_ext_uv.x, 1 - _ext_uv.y) : _ext_uv;
  _tm = _time;
  _normal = _ext_normal;
  gl_Position = effect();
}
