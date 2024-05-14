#version 330 core
out vec4 fragColor;

// Additional information for lighting
in vec4 normal_worldSpace;
in vec4 position_worldSpace;
in vec4 vColor;

uniform int wire = 0;


void main() {
    if (wire == 1) {
        fragColor = vec4(0.0, 0.0, 0.0, 1);
        return;
    }
    vec4 lightPos   = vec4(30.0, 0.0, 200.0, 1.0);
    vec3 lightColor = vec3(1.5f, 1.5f, 1.5f);
    vec4 lightDir   = normalize(-lightPos + position_worldSpace);
    float c = clamp(dot(-normal_worldSpace, lightDir), 0, 1);
    float k = 0.2;
    float r = vColor[0] * (1.0 - k) + (c * lightColor[0] * k);
    float g = vColor[1] * (1.0 - k) + (c * lightColor[1] * k);
    float b = vColor[2] * (1.0 - k) + (c * lightColor[2] * k);
    fragColor = vec4(r, g, b, 1.0);
    // fragColor = vec4(vColor[0] * c * lightColor[0], vColor[1] * c * lightColor[0], vColor[2] * c * lightColor[0], 1);
    // fragColor = vColor;
    // fragColor = vec4(normal_worldSpace[0], normal_worldSpace[1], normal_worldSpace[2], 1);
}
