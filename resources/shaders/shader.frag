#version 330 core
out vec4 fragColor;

// Additional information for lighting
in vec4 normal_worldSpace;
in vec4 position_worldSpace;

uniform float red = 1.0;
uniform float green = 1.0;
uniform float blue = 1.0;
uniform float alpha = 1.0;

void main() {
    vec4 lightPos   = vec4(0.0, 0.0, 200.0 , 1.0);
    vec3 lightColor = vec3(1.0f, alpha, 0.0f);
    vec4 lightDir   = normalize(-lightPos + position_worldSpace);
    float c = clamp(dot(-normal_worldSpace, lightDir), 0, 1);

    fragColor = vec4(red * c * lightColor[0], green * c * lightColor[0], blue * c * lightColor[0], 1);
}