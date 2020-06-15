vec4 effect() {
    ready_spritebatch();
    return _projection * _view * _model * _sb_model * vec4(_vertex.xy, 0, 1);
}
