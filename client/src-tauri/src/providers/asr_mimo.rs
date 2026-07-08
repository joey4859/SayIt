// 小米 MiMo ASR — mimo-v2.5-asr
// OpenAI chat/completions 兼容接口，支持 Base64 data URL 音频（wav/mp3）。
// 与千问的差异：鉴权头是 `api-key`（非 Bearer）、无 app_id、body 为 OpenAI chat 结构、
// 响应为 choices[0].message.content（纯字符串）。

use super::types::{AsrProviderConfig, AsrResult, TestResult};
use std::time::Instant;

const API_URL: &str = "https://api.xiaomimimo.com/v1/chat/completions";

/// 将 16kHz 单声道 16-bit PCM 封装为 WAV 容器（MiMo 只接受 wav/mp3，不接受裸 PCM）。
fn pcm_to_wav(pcm: &[u8], sr: u32) -> Vec<u8> {
    let ds = pcm.len() as u32;
    let mut w = Vec::with_capacity(44 + pcm.len());
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + ds).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&sr.to_le_bytes());
    w.extend_from_slice(&(sr * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&ds.to_le_bytes());
    w.extend_from_slice(pcm);
    w
}

/// 从供应商额外配置读取识别语言（auto|zh|en），默认 auto。
fn resolve_language(config: &AsrProviderConfig) -> String {
    config
        .extra
        .get("language")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("auto")
        .to_string()
}

fn build_body(data_url: &str, language: &str) -> serde_json::Value {
    serde_json::json!({
        "model": "mimo-v2.5-asr",
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "input_audio", "input_audio": { "data": data_url } }
                ]
            }
        ],
        "asr_options": { "language": language }
    })
}

/// 解析 OpenAI chat 响应中的转写文本：choices[0].message.content。
/// content 可能是字符串，也可能是分段数组，两种都兼容。
fn extract_text(data: &serde_json::Value) -> String {
    let content = data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"));
    match content {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|seg| {
                seg.get("text")
                    .and_then(|t| t.as_str())
                    .or_else(|| seg.as_str())
            })
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

pub async fn transcribe(
    audio_pcm_b64: &str,
    sample_rate: u32,
    config: &AsrProviderConfig,
    _hotwords: &[String],
) -> Result<AsrResult, String> {
    let pcm = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        audio_pcm_b64,
    )
    .map_err(|e| format!("base64 解码失败: {}", e))?;

    if pcm.is_empty() {
        return Ok(AsrResult { text: String::new(), elapsed_ms: 0 });
    }

    let wav = pcm_to_wav(&pcm, sample_rate);
    let wav_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav);
    let data_url = format!("data:audio/wav;base64,{}", wav_b64);
    let language = resolve_language(config);
    let body = build_body(&data_url, &language);

    let client = reqwest::Client::new();
    let start = Instant::now();

    let resp = client
        .post(API_URL)
        .header("api-key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("MiMo ASR 错误 {}: {}", status, &body[..body.len().min(300)]));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(AsrResult { text: extract_text(&data), elapsed_ms })
}

pub async fn test_connection(config: &AsrProviderConfig) -> TestResult {
    let silence = vec![0u8; 16000]; // 0.5s 静音
    let wav = pcm_to_wav(&silence, 16000);
    let wav_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav);
    let data_url = format!("data:audio/wav;base64,{}", wav_b64);
    let body = build_body(&data_url, "auto");

    let client = reqwest::Client::new();
    let start = Instant::now();

    let result = client
        .post(API_URL)
        .header("api-key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(resp) if resp.status().is_success() => TestResult {
            ok: true,
            message: format!("连接成功 ({}ms)", elapsed_ms),
            elapsed_ms,
            detail: String::new(),
        },
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            TestResult {
                ok: false,
                message: format!("API 错误 {}: {}", status, &body[..body.len().min(100)]),
                elapsed_ms,
                detail: String::new(),
            }
        }
        Err(e) => TestResult {
            ok: false,
            message: format!("连接失败: {}", e),
            elapsed_ms,
            detail: String::new(),
        },
    }
}
