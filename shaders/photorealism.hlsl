Texture2D SceneTexture : register(t0);
SamplerState SceneSampler : register(s0);

cbuffer SettingsBuffer : register(b0)
{
    float Exposure;
    float Contrast;
    float Saturation;
    float Temperature;
    float Tint;
    float HighlightRolloff;
    float InputNeedsSrgbDecode;
    float OutputNeedsSrgbEncode;
    float OutputMode;
    float HdrPaperWhiteNits;
    float HdrPeakNits;
    float Padding;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
};

VertexOutput VSMain(uint vertex_id : SV_VertexID)
{
    VertexOutput output;
    output.uv = float2((vertex_id << 1) & 2, vertex_id & 2);
    output.position = float4(output.uv * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
    return output;
}

float luminance(float3 color)
{
    return dot(color, float3(0.2126, 0.7152, 0.0722));
}

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

float3 pq_to_relative_linear(float3 color)
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
    return normalized_nits * 10000.0 / max(HdrPaperWhiteNits, 1.0);
}

float3 relative_linear_to_pq(float3 color)
{
    const float m1 = 0.1593017578;
    const float m2 = 78.84375;
    const float c1 = 0.8359375;
    const float c2 = 18.8515625;
    const float c3 = 18.6875;
    float3 normalized_nits =
        max(color, 0.0) * max(HdrPaperWhiteNits, 1.0) / 10000.0;
    float3 power = pow(normalized_nits, m1);
    return pow((c1 + c2 * power) / (1.0 + c3 * power), m2);
}

float3 decode_output(float3 color)
{
    if (OutputMode > 1.5)
    {
        return color * 80.0 / max(HdrPaperWhiteNits, 1.0);
    }
    if (OutputMode > 0.5)
    {
        return pq_to_relative_linear(color);
    }
    return InputNeedsSrgbDecode > 0.5 ? srgb_to_linear(color) : color;
}

float3 encode_output(float3 color)
{
    float peak = max(HdrPeakNits, HdrPaperWhiteNits);
    float relative_peak = peak / max(HdrPaperWhiteNits, 1.0);
    if (OutputMode > 1.5)
    {
        return clamp(color, 0.0, relative_peak) * HdrPaperWhiteNits / 80.0;
    }
    if (OutputMode > 0.5)
    {
        return saturate(relative_linear_to_pq(min(color, relative_peak)));
    }
    color = saturate(color);
    return OutputNeedsSrgbEncode > 0.5 ? linear_to_srgb(color) : color;
}

float3 apply_white_balance(float3 color)
{
    float temperature = clamp((Temperature - 6500.0) / 3500.0, -1.0, 1.0);
    float tint = clamp(Tint, -1.0, 1.0);
    float3 balance = float3(
        1.0 - 0.08 * temperature - 0.05 * tint,
        1.0 + 0.10 * tint,
        1.0 + 0.10 * temperature - 0.05 * tint);
    return color * balance / max(luminance(balance), 0.0001);
}

float3 apply_highlight_rolloff(float3 color, float limit)
{
    float amount = saturate(HighlightRolloff);
    float knee = limit * lerp(1.0, 0.5, amount);
    float headroom = max(limit - knee, 0.0001);
    float3 excess = max(color - knee, 0.0);
    float3 compressed = knee + headroom * (1.0 - exp(-excess / headroom));
    return min(color, compressed);
}

float3 apply_tonemap(float3 color)
{
    color = max(color * exp2(Exposure), 0.0);
    color = apply_white_balance(color);
    const float pivot = 0.18;
    color = pivot * pow(max(color, 0.000001) / pivot, Contrast);
    float luma = luminance(color);
    color = lerp(luma.xxx, color, Saturation);
    float peak = OutputMode > 0.5
        ? max(HdrPeakNits, HdrPaperWhiteNits) / max(HdrPaperWhiteNits, 1.0)
        : 1.0;
    return apply_highlight_rolloff(color, peak);
}

float4 PSMain(VertexOutput input) : SV_Target
{
    float4 source = SceneTexture.Sample(SceneSampler, input.uv);
    float3 color = decode_output(source.rgb);
    color = apply_tonemap(color);
    return float4(encode_output(color), source.a);
}
