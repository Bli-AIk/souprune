let a = select(0.0, 1.0, input.distance <= 0.0);
return vec4<f32>(input.color.rgb, a);
