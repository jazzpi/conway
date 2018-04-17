#version 130
in vec2 pos;
out vec4 dbg_color;
// 1: Default zoom level, 10: 10x larger cells, 0.1: 1/10th as large
uniform float zoom;
uniform ivec2 viewport;

#define DEFAULT_WIDTH_IN_PIXELS 600.0
#define DEFAULT_HEIGHT_IN_PIXELS 600.0
#define DEFAULT_WIDTH_IN_CELLS 20.0
#define DEFAULT_HEIGHT_IN_CELLS 20.0

void main() {
    float scale_x = DEFAULT_WIDTH_IN_PIXELS / viewport.x;
    float scale_y = DEFAULT_HEIGHT_IN_PIXELS / viewport.y;
    gl_Position = vec4(pos.x / DEFAULT_WIDTH_IN_CELLS * scale_x,
                       pos.y / DEFAULT_HEIGHT_IN_CELLS * scale_y,
                       0, zoom);
    dbg_color = vec4(pos.x,
                     pos.y,
                     0, 1);
    // dbg_color = vec4(scale_x, scale_y, 0, 1);
}
