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
    float3 LuminanceWeights;
    float WeightsPadding;
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
    return dot(color, LuminanceWeights);
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
    float3 color = decode_for_input(
        source.rgb, OutputMode, HdrPaperWhiteNits, InputNeedsSrgbDecode);
    color = apply_tonemap(color);
    float3 encoded = encode_for_output(
        color, OutputMode, HdrPaperWhiteNits, HdrPeakNits, OutputNeedsSrgbEncode);
    return float4(encoded, source.a);
}
