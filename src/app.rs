// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::fl;
use crate::weather::{self, WeatherData};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use std::time::Duration;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Current weather data
    weather_data: Option<WeatherData>,
    /// Loading state
    loading: bool,
    /// Error message if any
    error: Option<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    FetchWeather,
    WeatherFetched(Result<WeatherData, String>),
    UpdateCity(String),
    UpdateApiKey(String),
    UpdateLatitude(String),
    UpdateLongitude(String),
    ToggleAutoUpdate(bool),
    UpdateInterval(u64),
    UpdateUnits(String),
}

// Helper function to fetch weather data
async fn fetch_weather_data(lat: f64, lon: f64) -> Result<WeatherData, String> {
    match weather::get_weather_data(lat, lon).await {
        Ok(data) => Ok(data),
        Err(e) => Err(e.to_string()),
    }
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.marcos.CosmicWeather";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Construct the app model with the runtime's core.
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => {
                    config
                }
            })
            .unwrap_or_default();

        let app = AppModel {
            core,
            popup: None,
            config,
            weather_data: None,
            loading: false,
            error: None,
        };

        // Fetch weather data if coordinates are configured
        let mut task = Task::none();
        if let (Some(lat_str), Some(lon_str)) = (&app.config.latitude, &app.config.longitude) {
            if let (Ok(lat), Ok(lon)) = (lat_str.parse::<f64>(), lon_str.parse::<f64>()) {
                task = Task::perform(fetch_weather_data(lat, lon), Message::WeatherFetched).map(cosmic::Action::App);
            }
        }

        (app, task)
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        let icon_name = match &self.weather_data {
            Some(weather) => {
                // Map weather condition to appropriate icon
                match weather.icon.as_str() {
                    "01d" | "01n" => "weather-clear-symbolic", // clear sky
                    "02d" | "02n" => "weather-few-clouds-symbolic", // few clouds
                    "03d" | "03n" => "weather-clouds-symbolic", // scattered clouds
                    "04d" | "04n" => "weather-overcast-symbolic", // broken clouds
                    "09d" | "09n" => "weather-showers-symbolic", // shower rain
                    "10d" | "10n" => "weather-showers-symbolic", // rain
                    "11d" | "11n" => "weather-storm-symbolic", // thunderstorm
                    "13d" | "13n" => "weather-snow-symbolic", // snow
                    "50d" | "50n" => "weather-fog-symbolic", // mist
                    _ => "weather-severe-alert-symbolic",
                }
            },
            None => "weather-severe-alert-symbolic", // Default to alert icon when no weather data
        };

        let icon = widget::icon::from_name(icon_name)
            .size(self.core.applet.suggested_size(true).0)
            .symbolic(true);

        let temperature_text = match &self.weather_data {
            Some(weather) => format!("{}°C", weather.temperature as i32),
            None => {
                if self.loading {
                    "...".to_string()
                } else {
                    "?".to_string()
                }
            }
        };

        let temperature = self.core.applet.text(temperature_text);

        // Convert to Element to make both options compatible
        let content: Element<Self::Message> = if self.core.applet.is_horizontal() {
            widget::row()
                .push(icon)
                .push(temperature)
                .align_y(cosmic::iced::alignment::Vertical::Center)
                .spacing(4)
                .into()
        } else {
            widget::column()
                .push(icon)
                .push(temperature)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .spacing(4)
                .into()
        };

        let button = widget::button::custom(content)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press(Message::TogglePopup);

        button.into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut content_list = widget::list_column()
            .padding(5)
            .spacing(10);

        // Show weather data if available
        if let Some(weather) = &self.weather_data {
            let weather_info = widget::list_column()
                .padding(10)
                .spacing(5)
                .add(widget::text::title3(&weather.location))
                .add(widget::text::heading(format!("{}°C", weather.temperature as i32)))
                .add(widget::text(format!("Feels like {}°C", weather.feels_like as i32)))
                .add(widget::text(&weather.description))
                .add(widget::text(format!("Humidity: {}%", weather.humidity)));

            content_list = content_list.add(weather_info);
        } else if self.loading {
            content_list = content_list.add(widget::text("Loading weather..."));
        } else if let Some(error) = &self.error {
            content_list = content_list.add(widget::text::body(format!("Error: {}", error)));
        } else {
            content_list = content_list.add(widget::text("No weather data available"));
        }

        // Add refresh button
        let refresh_button = widget::button::standard(fl!("refresh"))
            .on_press(Message::FetchWeather);

        content_list = content_list.add(refresh_button);

        // Add settings section
        let settings_section = widget::list_column()
            .padding(10)
            .spacing(10)
            .add(widget::settings::item::builder(fl!("latitude")).control(
                widget::text_input(fl!("latitude-placeholder"), self.config.latitude.as_deref().unwrap_or(""))
                    .on_input(|input| Message::UpdateLatitude(input))
            ))
            .add(widget::settings::item::builder(fl!("longitude")).control(
                widget::text_input(fl!("longitude-placeholder"), self.config.longitude.as_deref().unwrap_or(""))
                    .on_input(|input| Message::UpdateLongitude(input))
            ))
            .add(widget::settings::item::builder(fl!("city")).control(
                widget::text_input("", self.config.city.as_deref().unwrap_or(""))
                    .on_input(Message::UpdateCity)
            ))
            .add(widget::settings::item::builder(fl!("units")).control(
                widget::dropdown(&["Celsius", "Fahrenheit"],
                    match self.config.units.as_str() {
                        "imperial" => Some(1),
                        _ => Some(0), // default to Celsius
                    },
                    |i| {
                        if i == 1 {
                            Message::UpdateUnits("imperial".to_string())
                        } else {
                            Message::UpdateUnits("metric".to_string())
                        }
                    })
            ))
            .add(widget::settings::item::builder(fl!("auto-update")).control(
                widget::toggler(self.config.auto_update).on_toggle(Message::ToggleAutoUpdate)
            ));

        content_list = content_list.add(settings_section);

        self.core.applet.popup_container(content_list).into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-lived async tasks running in the background which
    /// emit messages to the application through a channel. They may be conditionally
    /// activated by selectively appending to the subscription batch, and will
    /// continue to execute for the duration that they remain in the batch.
    fn subscription(&self) -> Subscription<Self::Message> {
        use cosmic::iced::time;

        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    Message::UpdateConfig(update.config)
                }),
        ];

        // Add periodic update subscription if auto-update is enabled
        if self.config.auto_update &&
           self.config.latitude.is_some() &&
           self.config.longitude.is_some() {
            let update_interval = std::cmp::max(self.config.update_interval, 5); // Minimum 5 minutes
            subscriptions.push(
                time::every(Duration::from_secs(update_interval * 60))
                    .map(|_| Message::FetchWeather)
            );
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime. The application will not exit until all
    /// tasks are finished.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SubscriptionChannel => {
                // For example purposes only.
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::FetchWeather => {
                if let (Some(lat_str), Some(lon_str)) = (&self.config.latitude, &self.config.longitude) {
                    if let (Ok(lat), Ok(lon)) = (lat_str.parse::<f64>(), lon_str.parse::<f64>()) {
                        self.loading = true;
                        self.error = None;
                        return Task::perform(
                            fetch_weather_data(lat, lon),
                            Message::WeatherFetched
                        ).map(cosmic::Action::App);
                    }
                }
            }
            Message::WeatherFetched(result) => {
                self.loading = false;
                match result {
                    Ok(weather_data) => {
                        self.weather_data = Some(weather_data);
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }
            Message::UpdateCity(city) => {
                let mut config = self.config.clone();
                config.city = Some(city);
                self.config = config;

                // Save the new configuration
                if let Ok(helper) = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    if let Err(err) = self.config.write_entry(&helper) {
                        eprintln!("Error saving config: {}", err);
                    }
                }
            }
            Message::UpdateApiKey(_api_key) => {
                // In the MET Norway API, we don't need an API key
                // But we keep this message for compatibility
            }
            Message::UpdateLatitude(lat) => {
                let mut config = self.config.clone();
                config.latitude = Some(lat);
                self.config = config;

                // Save the new configuration
                if let Ok(helper) = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    if let Err(err) = self.config.write_entry(&helper) {
                        eprintln!("Error saving config: {}", err);
                    }
                }
            }
            Message::UpdateLongitude(lon) => {
                let mut config = self.config.clone();
                config.longitude = Some(lon);
                self.config = config;

                // Save the new configuration
                if let Ok(helper) = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    if let Err(err) = self.config.write_entry(&helper) {
                        eprintln!("Error saving config: {}", err);
                    }
                }
            }
            Message::ToggleAutoUpdate(enabled) => {
                let mut config = self.config.clone();
                config.auto_update = enabled;
                self.config = config;

                // Save the new configuration
                if let Ok(helper) = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    if let Err(err) = self.config.write_entry(&helper) {
                        eprintln!("Error saving config: {}", err);
                    }
                }
            }
            Message::UpdateUnits(units) => {
                let mut config = self.config.clone();
                config.units = units;
                self.config = config;

                // Save the new configuration
                if let Ok(helper) = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    if let Err(err) = self.config.write_entry(&helper) {
                        eprintln!("Error saving config: {}", err);
                    }
                }
            }
            Message::UpdateInterval(interval) => {
                let mut config = self.config.clone();
                config.update_interval = interval;
                self.config = config;

                // Save the new configuration
                if let Ok(helper) = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    if let Err(err) = self.config.write_entry(&helper) {
                        eprintln!("Error saving config: {}", err);
                    }
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}