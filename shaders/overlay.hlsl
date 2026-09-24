struct OverlayVertex
{
    float4 position_uv;
    float4 color;
    float4 shape;
};

StructuredBuffer<OverlayVertex> OverlayVertices : register(t0);

cbuffer OverlayBuffer : register(b0)
{
    float OutputMode;
    float HdrPaperWhiteNits;
    float HdrPeakNits;
    float OutputNeedsSrgbEncode;
};

Texture2D FontTexture : register(t0);
SamplerState FontSampler : register(s0);

struct OverlayPixel
{
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
    float kind : TEXCOORD1;
};

OverlayPixel VSOverlay(uint vertex_id : SV_VertexID)
{
    OverlayVertex source = OverlayVertices[vertex_id];

    OverlayPixel output;
    output.position = float4(source.position_uv.xy, 0.0, 1.0);
    output.uv = source.position_uv.zw;
    output.color = source.color;
    output.kind = source.shape.x;
    return output;
}

float4 PSOverlay(OverlayPixel input) : SV_Target
{
    float alpha = input.color.a;
    if (input.kind > 0.5)
    {
        alpha *= FontTexture.Sample(FontSampler, input.uv).r;
    }
    float3 encoded = encode_for_output(
        input.color.rgb,
        OutputMode,
        HdrPaperWhiteNits,
        HdrPeakNits,
        OutputNeedsSrgbEncode);
    return float4(encoded, alpha);
}
