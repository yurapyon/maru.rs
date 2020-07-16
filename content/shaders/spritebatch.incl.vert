vec3 effect() {
    ready_spritebatch();
    return _screen * _view * _model * _sb_model * vec3(_ext_vertex, 1.0);
}
