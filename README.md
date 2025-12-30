# Cosmic Weather

A weather applet for the COSMIC™ desktop environment.

## About

Cosmic Weather is a simple and functional weather applet for the COSMIC™ desktop environment. It displays current weather conditions directly in the panel, with temperature and an icon representing the conditions.

## Features

- Displays current temperature next to the icon in the panel
- Representative weather condition icons
- Free weather data from MET Norway API (no API key required)
- Multi-language support (Portuguese and English)
- Coordinate configuration (latitude and longitude)
- Automatic and manual updates

## Installation

To install Cosmic Weather:

```bash
# Clone the repository
git clone https://github.com/marcos/cosmic-weather.git
cd cosmic-weather

# Build the project
cargo build --release

# Install the applet
just install  
```

## Configuration

1. Click on the applet in the panel
2. Enter the latitude and longitude of your location
3. Click "Refresh" to get weather data
4. Temperature will be displayed next to the icon in the panel

### Example coordinates:
- Caxias do sul,RS Latitude -29.1629, Longitude -51.1833
- São Paulo, SP: Latitude: -23.5505, Longitude: -46.6333
- Rio de Janeiro, RJ: Latitude: -22.9068, Longitude: -43.1729

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.# cosmic-weather
