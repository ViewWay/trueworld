// services/ai-inference/src/inference.rs

use std::collections::HashMap;
use anyhow::Result;

use super::config::InferenceConfig;

/// AI 推理引擎
pub struct InferenceEngine {
    config: InferenceConfig,
    models: HashMap<String, Box<dyn Model>>,
}

impl InferenceEngine {
    pub async fn new(config: InferenceConfig) -> Result<Self> {
        let mut engine = Self {
            config: config.clone(),
            models: HashMap::new(),
        };

        // 加载模型
        engine.load_models().await?;

        Ok(engine)
    }

    async fn load_models(&mut self) -> Result<()> {
        // 加载 Whisper 模型
        use super::models::WhisperModel;
        let whisper = Box::new(WhisperModel::new("tiny").await?);
        self.models.insert("whisper-tiny".to_string(), whisper);

        // 加载 Qwen 模型 (用于意图分类)
        use super::models::QwenModel;
        let qwen = Box::new(QwenModel::new("0.5b").await?);
        self.models.insert("qwen-0.5b".to_string(), qwen);

        Ok(())
    }

    pub async fn transcribe(&mut self, audio: &[f32], language: &str, model: &str) -> Result<Transcription> {
        let model_key = format!("whisper-{}", model);
        let model = self.models.get_mut(&model_key)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_key))?;

        model.transcribe(audio, language).await
    }

    pub async fn classify_intent(&mut self, text: &str, context: Option<serde_json::Value>) -> Result<IntentClassification> {
        let model = self.models.get_mut("qwen-0.5b")
            .ok_or_else(|| anyhow::anyhow!("Qwen model not loaded"))?;

        model.classify_intent(text, context).await
    }

    pub async fn recognize_action(&mut self, poses: &[serde_json::Value], context: Option<serde_json::Value>) -> Result<ActionRecognition> {
        // 使用 LLM 进行动作识别
        let model = self.models.get_mut("qwen-0.5b")
            .ok_or_else(|| anyhow::anyhow!("Qwen model not loaded"))?;

        model.recognize_action(poses, context).await
    }
}

/// Model trait
#[async_trait::async_trait]
pub trait Model: Send + Sync {
    async fn transcribe(&mut self, audio: &[f32], language: &str) -> Result<Transcription>;
    async fn classify_intent(&mut self, text: &str, context: Option<serde_json::Value>) -> Result<IntentClassification>;
    async fn recognize_action(&mut self, poses: &[serde_json::Value], context: Option<serde_json::Value>) -> Result<ActionRecognition>;
}

/// 转录结果
#[derive(Debug, Clone)]
pub struct Transcription {
    pub text: String,
    pub confidence: f32,
    pub language: String,
}

/// 意图分类结果
#[derive(Debug, Clone)]
pub struct IntentClassification {
    pub intent: String,
    pub entities: Vec<Entity>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub label: String,
    pub confidence: f32,
}

/// 动作识别结果
#[derive(Debug, Clone)]
pub struct ActionRecognition {
    pub action: String,
    pub skill_match: Option<String>,
    pub confidence: f32,
    pub reasoning: Option<String>,
}
