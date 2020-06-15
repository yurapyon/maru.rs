#version 330 core

// basic
in vec2 _uv_coord;
in float _tm;
uniform sampler2D _diffuse;
// TODO
// unifrom sampler2D _normal;
uniform vec4 _base_color;

// spritebatch
in vec4 _sb_color;
in vec2 _sb_uv;

out vec4 _out_color;

float _time;

@

vec4 effect() {
  return _base_color * texture2D(_diffuse, _uv_coord);
}

@

void main() {
  _time = _tm;
  _out_color = effect();
}
