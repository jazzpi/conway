#version 130
in vec2 pos;
out vec4 dbg_color;
uniform float zoom;
uniform ivec2 viewport;

#define DEFAULT_WIDTH 600.0
#define DEFAULT_HEIGHT 600.0

void main() {
    float scale_x = DEFAULT_WIDTH / viewport.x;
    float scale_y = DEFAULT_HEIGHT / viewport.y;
    gl_Position = vec4(pos.x * scale_x, pos.y * scale_y, 0, zoom);
    dbg_color = vec4(scale_x, scale_y, 0, 1);
}
