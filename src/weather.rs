// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub feels_like: f64, // For MET Norway, this might be the same as temperature
    pub humidity: u8,
    pub description: String,
    pub icon: String,
    pub location: String,
    pub timestamp: std::time::SystemTime,
}

// MET Norway API structures
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Geometry {
    pub r#type: String,
    pub coordinates: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Units {
    pub air_pressure_at_sea_level: Option<String>,
    pub air_temperature: Option<String>,
    pub air_temperature_max: Option<String>,
    pub air_temperature_min: Option<String>,
    pub cloud_area_fraction: Option<String>,
    pub precipitation_amount: Option<String>,
    pub relative_humidity: Option<String>,
    pub wind_from_direction: Option<String>,
    pub wind_speed: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Meta {
    pub updated_at: DateTime<Local>,
    pub units: Units,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Details {
    pub air_pressure_at_sea_level: Option<f64>,
    pub air_temperature: Option<f64>,
    pub air_temperature_max: Option<f64>,
    pub air_temperature_min: Option<f64>,
    pub cloud_area_fraction: Option<f64>,
    pub relative_humidity: Option<f64>,
    pub wind_from_direction: Option<f64>,
    pub wind_speed: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Summary {
    pub symbol_code: String, // This will be used for weather icon
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Next1Hour {
    pub summary: Summary,
    pub details: Option<Details>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Instant {
    pub details: Details,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Data {
    pub instant: Instant,
    pub next_1_hours: Option<Next1Hour>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Timeseries {
    pub time: DateTime<Local>,
    pub data: Data,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Properties {
    pub meta: Meta,
    pub timeseries: Vec<Timeseries>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MetWeatherResponse {
    pub r#type: String,
    pub geometry: Geometry,
    pub properties: Properties,
}

pub async fn get_weather_data(lat: f64, lon: f64) -> Result<WeatherData, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={}&lon={}",
        lat, lon
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "cosmic-weather/1.0.0") // Required by MET Norway API
        .send()
        .await?;

    if response.status().is_success() {
        let weather_response: MetWeatherResponse = response.json().await?;

        // Get the first timeseries data (current weather)
        if let Some(timeseries) = weather_response.properties.timeseries.first() {
            let details = &timeseries.data.instant.details;

            // Extract weather data
            let temperature = details.air_temperature.unwrap_or(0.0);
            let humidity = details.relative_humidity.unwrap_or(0.0) as u8;
            let description = timeseries.data.next_1_hours.as_ref()
                .map(|h| h.summary.symbol_code.clone())
                .unwrap_or_else(|| "clear sky".to_string());

            // Map MET Norway weather codes to common descriptions
            let description_text = map_weather_code_to_description(&description);

            // Map MET Norway weather codes to icon codes
            let icon = map_weather_code_to_icon(&description);

            let weather_data = WeatherData {
                temperature,
                feels_like: temperature, // MET Norway doesn't provide feels_like, using temperature
                humidity,
                description: description_text,
                icon,
                location: format!("({}, {})", lat, lon), // For now, using coordinates as location
                timestamp: std::time::SystemTime::now(),
            };

            Ok(weather_data)
        } else {
            Err("No weather data available".into())
        }
    } else {
        Err(format!("API request failed with status: {}", response.status()).into())
    }
}

// Helper function to map MET Norway weather codes to descriptions
fn map_weather_code_to_description(code: &str) -> String {
    match code {
        "clearsky_day" | "clearsky_night" | "clearsky_polartwilight" => "Clear sky".to_string(),
        "fair_day" | "fair_night" | "fair_polartwilight" => "Fair".to_string(),
        "cloudy" | "partlycloudy_day" | "partlycloudy_night" | "partlycloudy_polartwilight" => "Cloudy".to_string(),
        "fog" => "Fog".to_string(),
        "lightrain" | "rainshowers_day" | "rainshowers_night" | "rainshowers_polartwilight" => "Light rain".to_string(),
        "rain" | "heavyrain" => "Rain".to_string(),
        "lightrainshowers_day" | "lightrainshowers_night" | "lightrainshowers_polartwilight" => "Light rain showers".to_string(),
        "lightsnow" => "Light snow".to_string(),
        "sleet" => "Sleet".to_string(),
        "lightsleet" => "Light sleet".to_string(),
        "thunderstorm" => "Thunderstorm".to_string(),
        "sleetshowers_day" | "sleetshowers_night" | "sleetshowers_polartwilight" => "Sleet showers".to_string(),
        "snowshowers_day" | "snowshowers_night" | "snowshowers_polartwilight" => "Snow showers".to_string(),
        "snow" | "heavysnow" => "Snow".to_string(),
        _ => "Unknown".to_string(),
    }
}

// Helper function to map MET Norway weather codes to icon codes
fn map_weather_code_to_icon(code: &str) -> String {
    match code {
        "clearsky_day" => "01d".to_string(),
        "clearsky_night" => "01n".to_string(),
        "fair_day" | "partlycloudy_day" => "02d".to_string(),
        "fair_night" | "partlycloudy_night" => "02n".to_string(),
        "cloudy" => "03d".to_string(), // Using same for day/night
        "rain" | "lightrain" | "rainshowers_day" | "rainshowers_night" | "rainshowers_polartwilight" => "10d".to_string(),
        "snow" | "heavysnow" | "snowshowers_day" | "snowshowers_night" | "snowshowers_polartwilight" => "13d".to_string(),
        "fog" => "50d".to_string(),
        "thunderstorm" => "11d".to_string(),
        "sleet" | "lightsleet" => "09d".to_string(),
        _ => "01d".to_string(), // Default to clear sky icon
    }
}