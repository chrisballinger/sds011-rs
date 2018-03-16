use chrono::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMeasurement {
    pub timestamp: DateTime<Utc>,
    pub pm2_5: f32,
    pub pm10: f32
}
