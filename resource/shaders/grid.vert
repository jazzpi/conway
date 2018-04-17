#version 130
in vec2 pos;
uniform float zoom;
uniform ivec2 viewport;

#define DEFAULT_WIDTH 600.0
#define DEFAULT_HEIGHT 600.0
#define DEFAULT_WIDTH_IN_CELLS 20.0
#define DEFAULT_HEIGHT_IN_CELLS 20.0

void main() {
    float scale_x = DEFAULT_WIDTH / viewport.x;
    float scale_y = DEFAULT_HEIGHT / viewport.y;
    gl_Position = vec4(pos.x / DEFAULT_WIDTH_IN_CELLS * scale_x,
                       pos.y / DEFAULT_HEIGHT_IN_CELLS * scale_y,
                       0.5, zoom);
}
