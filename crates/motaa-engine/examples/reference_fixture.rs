use motaa_engine::{ChartRequest, calculate};
fn main() {
    let r = ChartRequest {
        timezone_id: None,
        timezone_offset_minutes: 0,
        year: 2000,
        month: 1,
        day: 1,
        hour: 12,
        minute: 0,
        second: 0,
        latitude: 0.0,
        longitude: 0.0,
        ..ChartRequest::default()
    };
    let chart = calculate(&r).expect("valid chart");
    println!(
        "{}",
        serde_json::to_string_pretty(&chart).expect("serialize chart")
    );
}
