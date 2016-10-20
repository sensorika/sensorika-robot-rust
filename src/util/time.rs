use chrono::*;

pub fn now() -> f64{
    let local: DateTime<Local> = Local::now();
    local.timestamp() as f64 + (local.timestamp_subsec_nanos() as f64 * 1e-9)
}