#version 330 core

layout (location = 0) in vec2 _ext_vertex;
layout (location = 1) in vec2 _ext_uv;

layout (location = 3) in vec4  _ext_sb_uv;
layout (location = 4) in vec2  _ext_sb_position;
layout (location = 5) in vec2  _ext_sb_scale;
layout (location = 6) in float _ext_sb_rotation;
layout (location = 7) in vec4  _ext_sb_color;

// basic
uniform mat3 _screen;
uniform mat3 _view;
uniform mat3 _model;
uniform float _time;
uniform int _flip_uvs;

out vec2 _uv_coord;
out float _tm;
out vec3 _normal;

// spritebatch
mat3 _sb_model;
out vec4 _sb_color;
out vec2 _sb_uv;

mat3 mat3_from_transform2d(float x, float y, float sx, float sy, float r) {
    mat3 ret = mat3(1.0);
    float rc = cos(r);
    float rs = sin(r);
    ret[0][0] =  rc * sx;
    ret[0][1] =  rs * sx;
    ret[1][0] = -rs * sy;
    ret[1][1] =  rc * sy;
    ret[2][0] = x;
    ret[2][1] = y;
    return ret;
}

void ready_spritebatch() {
    // TODO flip uvs
    // multiply sb uv by the uv from mesh
    //   normal mesh uvs can still be used with sb

    // generate actual uv point for each vertex
    //   based on input rect
    // _ext_sb_uv => vec4(x1 y1 x2 y2)
    switch (gl_VertexID) {
    case 0:
        _sb_uv = _ext_sb_uv.zw;
        break;
    case 1:
        _sb_uv = _ext_sb_uv.zy;
        break;
    case 2:
        _sb_uv = _ext_sb_uv.xw;
        break;
    case 3:
        _sb_uv = _ext_sb_uv.xy;
        break;
    }

    _sb_color = _ext_sb_color;
    _sb_model = mat3_from_transform2d(_ext_sb_position.x,
                                      _ext_sb_position.y,
                                      _ext_sb_scale.x,
                                      _ext_sb_scale.y,
                                      _ext_sb_rotation);
}

@

vec3 effect() {
  // return _projection * _view * _model * vec4(_ext_vertex, 0.0, 1.0);
  mat3 proj = mat3(1.0);
  proj[0][0] = 1./300.;
  proj[1][1] = 1./200.;
  proj[2][0] = -1;
  proj[2][1] = -1;
  return proj * vec3(_ext_vertex, 1.0);
}

@

void main() {
  _uv_coord = _flip_uvs != 0 ? vec2(_ext_uv.x, 1 - _ext_uv.y) : _ext_uv;
  _tm = _time;
  gl_Position = vec4(effect(), 1.0);
}
