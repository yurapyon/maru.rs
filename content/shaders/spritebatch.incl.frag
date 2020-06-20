vec4 effect() {
    // return vec4(1, 1, 1,1);
    // return vec4(_sb_color.xyz, 1.);
    // return vec4(_sb_uv.x, _sb_uv.y, 1, 1);
    // return texture2D(_tx_diffuse, _sb_uv);
    return _base_color * _sb_color * texture2D(_tx_diffuse, _sb_uv);
}
