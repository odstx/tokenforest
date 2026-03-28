use lazy_static::lazy_static;
use prometheus::{IntCounterVec, Opts, Registry};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref INPUT_TOKENS: IntCounterVec = IntCounterVec::new(
        Opts::new("tokenforest_input_tokens_total", "Total number of input tokens processed"),
        &["model", "api_key_id"]
    ).expect("Failed to create input_tokens counter");
    
    pub static ref OUTPUT_TOKENS: IntCounterVec = IntCounterVec::new(
        Opts::new("tokenforest_output_tokens_total", "Total number of output tokens generated"),
        &["model", "api_key_id"]
    ).expect("Failed to create output_tokens counter");
    
    pub static ref REQUESTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("tokenforest_requests_total", "Total number of API requests"),
        &["model", "api_key_id", "status"]
    ).expect("Failed to create requests_total counter");
}

pub fn init() {
    REGISTRY.register(Box::new(INPUT_TOKENS.clone())).expect("Failed to register INPUT_TOKENS");
    REGISTRY.register(Box::new(OUTPUT_TOKENS.clone())).expect("Failed to register OUTPUT_TOKENS");
    REGISTRY.register(Box::new(REQUESTS_TOTAL.clone())).expect("Failed to register REQUESTS_TOTAL");
}

pub fn record_tokens(model: &str, api_key_id: &str, input_tokens: u64, output_tokens: u64) {
    INPUT_TOKENS.with_label_values(&[model, api_key_id]).inc_by(input_tokens);
    OUTPUT_TOKENS.with_label_values(&[model, api_key_id]).inc_by(output_tokens);
}

pub fn record_request(model: &str, api_key_id: &str, status: &str) {
    REQUESTS_TOTAL.with_label_values(&[model, api_key_id, status]).inc();
}

pub fn export() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).expect("Failed to encode metrics");
    String::from_utf8(buffer).expect("Failed to convert metrics to string")
}
