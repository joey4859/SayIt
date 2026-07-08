// Model catalog

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSource {
    pub source: String,
    pub files: Vec<ModelFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    pub name: String,
    pub url: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model_type: String,
    pub total_size_bytes: u64,
    pub languages: Vec<String>,
    pub sources: Vec<DownloadSource>,
    #[serde(default)]
    pub archive_url: Option<String>,
    /// 速度评级 0–10（可带小数，10 最快）
    #[serde(default)]
    pub speed: f32,
    /// 准确度评级 0–10（可带小数，10 最准）
    #[serde(default)]
    pub accuracy: f32,
    /// 是否为推荐（默认）模型
    #[serde(default)]
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub total_size_bytes: u64,
    pub path: String,
    pub complete: bool,
}

fn hf(repo: &str, file: &str) -> String {
    format!("https://huggingface.co/{}/resolve/main/{}", repo, file)
}

fn hf_mirror(repo: &str, file: &str) -> String {
    format!("https://hf-mirror.com/{}/resolve/main/{}", repo, file)
}

fn ms(repo: &str, file: &str) -> String {
    format!("https://modelscope.cn/models/{}/resolve/master/{}", repo, file)
}

/// Qwen3-ASR ONNX 的 ModelScope 源（zengshuishui/Qwen3-ASR-onnx，含 model_0.6B int8 + 共享 tokenizer）
fn qwen3_ms(file: &str) -> String {
    ms("zengshuishui/Qwen3-ASR-onnx", file)
}

pub fn get_available_models() -> Vec<ModelInfo> {
    let hf_repo = "csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17";
    let ms_repo = "xiaowangge/sherpa-onnx-sense-voice-small";
    let langs = vec!["zh".into(), "en".into(), "ja".into(), "ko".into(), "yue".into()];

    vec![
        ModelInfo {
            id: "sensevoice-small".into(),
            name: "SenseVoice Small (INT8)".into(),
            description: "内存占用 600M 左右 · 中英日韩粤 · 速度极快，日常首选".into(),
            model_type: "sensevoice".into(),
            total_size_bytes: 228 * 1024 * 1024,
            speed: 8.5,
            accuracy: 7.0,
            recommended: true,
            languages: langs.clone(),
            sources: vec![
                DownloadSource {
                    source: "ModelScope".into(),
                    files: vec![
                        ModelFile { name: "model.int8.onnx".into(), url: ms(ms_repo, "model_q8.onnx"), size_bytes: 0, sha256: None },
                        ModelFile { name: "tokens.txt".into(), url: ms(ms_repo, "tokens.txt"), size_bytes: 0, sha256: None },
                    ],
                },
                DownloadSource {
                    source: "HuggingFace".into(),
                    files: vec![
                        ModelFile { name: "model.int8.onnx".into(), url: hf(hf_repo, "model.int8.onnx"), size_bytes: 0, sha256: None },
                        ModelFile { name: "tokens.txt".into(), url: hf(hf_repo, "tokens.txt"), size_bytes: 0, sha256: None },
                    ],
                },
                DownloadSource {
                    source: "HuggingFace Mirror".into(),
                    files: vec![
                        ModelFile { name: "model.int8.onnx".into(), url: hf_mirror(hf_repo, "model.int8.onnx"), size_bytes: 0, sha256: None },
                        ModelFile { name: "tokens.txt".into(), url: hf_mirror(hf_repo, "tokens.txt"), size_bytes: 0, sha256: None },
                    ],
                },
            ],
            archive_url: None,
        },
        // ── Qwen3-ASR 0.6B（speech-LLM，52 语言+方言+热词）──
        ModelInfo {
            id: "qwen3-asr-0.6b".into(),
            name: "Qwen3-ASR 0.6B (INT8)".into(),
            description: "内存占用 2.3G 左右 · 多语种支持 · 速度较慢".into(),
            model_type: "qwen3-asr".into(),
            total_size_bytes: 940 * 1024 * 1024,
            speed: 3.0,
            accuracy: 7.5,
            recommended: false,
            languages: vec!["zh".into(), "en".into(), "yue".into(), "ja".into(), "ko".into()],
            // ModelScope 多文件下载（国内原生、无需 GitHub 代理），与官方 tar 同源同模型
            sources: vec![
                DownloadSource {
                    source: "ModelScope".into(),
                    files: vec![
                        ModelFile { name: "conv_frontend.onnx".into(), url: qwen3_ms("model_0.6B/conv_frontend.onnx"), size_bytes: 0, sha256: None },
                        ModelFile { name: "encoder.int8.onnx".into(), url: qwen3_ms("model_0.6B/encoder.int8.onnx"), size_bytes: 0, sha256: None },
                        ModelFile { name: "decoder.int8.onnx".into(), url: qwen3_ms("model_0.6B/decoder.int8.onnx"), size_bytes: 0, sha256: None },
                        ModelFile { name: "tokenizer/merges.txt".into(), url: qwen3_ms("tokenizer/merges.txt"), size_bytes: 0, sha256: None },
                        ModelFile { name: "tokenizer/vocab.json".into(), url: qwen3_ms("tokenizer/vocab.json"), size_bytes: 0, sha256: None },
                        ModelFile { name: "tokenizer/tokenizer_config.json".into(), url: qwen3_ms("tokenizer/tokenizer_config.json"), size_bytes: 0, sha256: None },
                    ],
                },
            ],
            archive_url: None,
        },
    ]
}
