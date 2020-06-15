vec4 effect() {
    return _base_color * _sb_color * texture2D(_diffuse, _sb_uv);
}
