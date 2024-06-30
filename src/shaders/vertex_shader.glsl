#version 330 core
in vec2 position;
in vec3 color;
out vec3 vertexColor;
uniform mat4 MVP;
void main() {
    gl_Position = MVP * vec4(position, 0.0, 1.0);
    vertexColor = color;
}