float3 srgb_to_linear(float3 color)
{
    float3 low = color / 12.92;
    float3 high = pow(max((color + 0.055) / 1.055, 0.0), 2.4);
    return lerp(high, low, step(color, 0.04045));
}

float3 linear_to_srgb(float3 color)
{
    color = max(color, 0.0);
    float3 low = color * 12.92;
    float3 high = 1.055 * pow(color, 1.0 / 2.4) - 0.055;
    return lerp(high, low, step(color, 0.0031308));
}

float3 pq_to_relative_linear(float3 color, float paper_white)
{
    const float m1 = 0.1593017578;
    const float m2 = 78.84375;
    const float c1 = 0.8359375;
    const float c2 = 18.8515625;
    const float c3 = 18.6875;
    float3 power = pow(saturate(color), 1.0 / m2);
    float3 normalized_nits = pow(
        max((power - c1) / max(c2 - c3 * power, 0.000001), 0.0),
        1.0 / m1);
    return normalized_nits * 10000.0 / max(paper_white, 1.0);
}

float3 relative_linear_to_pq(float3 color, float paper_white)
{
    const float m1 = 0.1593017578;
    const float m2 = 78.84375;
    const float c1 = 0.8359375;
    const float c2 = 18.8515625;
    const float c3 = 18.6875;
    float3 normalized_nits = max(color, 0.0) * max(paper_white, 1.0) / 10000.0;
    float3 power = pow(normalized_nits, m1);
    return pow((c1 + c2 * power) / (1.0 + c3 * power), m2);
}

float3 decode_for_input(
    float3 color,
    float output_mode,
    float paper_white,
    float needs_srgb_decode)
{
    if (output_mode > 1.5)
    {
        return color * 80.0 / max(paper_white, 1.0);
    }
    if (output_mode > 0.5)
    {
        return pq_to_relative_linear(color, paper_white);
    }
    return needs_srgb_decode > 0.5 ? srgb_to_linear(color) : color;
}

float3 encode_for_output(
    float3 color,
    float output_mode,
    float paper_white,
    float peak_nits,
    float needs_srgb_encode)
{
    float peak = max(peak_nits, paper_white);
    float relative_peak = peak / max(paper_white, 1.0);
    if (output_mode > 1.5)
    {
        return clamp(color, 0.0, relative_peak) * paper_white / 80.0;
    }
    if (output_mode > 0.5)
    {
        return saturate(relative_linear_to_pq(min(color, relative_peak), paper_white));
    }
    color = saturate(color);
    return needs_srgb_encode > 0.5 ? linear_to_srgb(color) : color;
}
