#version 130
in vec4 dbg_color;
out vec4 color;

void main() {
    // color = vec4(0.2, 0.3, 0.8, 1);
    color = dbg_color;
}
