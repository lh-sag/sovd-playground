# OpenSOVD Web Interface Assets

This directory contains the web interface for the OpenSOVD gateway server, including all necessary resources stored locally for offline capability and improved security.

## Files Structure

```
assets/
├── index.html                 # Main web interface (single-page Vue.js application)
├── css/                       # Stylesheets directory
│   ├── bootstrap.min.css      # Bootstrap 5.3.0 CSS framework
│   ├── bootstrap-icons.css    # Bootstrap Icons 1.11.0 icon library styles
│   └── app.css                # Application-specific styles
├── js/                        # JavaScript directory
│   ├── bootstrap.bundle.min.js # Bootstrap 5.3.0 JavaScript bundle (includes Popper.js)
│   ├── vue.global.js          # Vue.js 3.3.4 framework
│   └── app.js                 # Application JavaScript (Vue app, telemetry, utilities)
└── fonts/                     # Icon font files
    ├── bootstrap-icons.woff   # Bootstrap Icons font (WOFF format)
    └── bootstrap-icons.woff2  # Bootstrap Icons font (WOFF2 format)
```

## Features

The web interface provides:

- **Component Explorer**: Browse and inspect SOVD components and their resources
- **Resource Viewer**: View diagnostic data, configurations, faults, and operations
- **API Version Selector**: Switch between different SOVD API versions
- **Dark/Light Theme**: Automatic theme based on system preferences
- **Search & Filter**: Quick component filtering with debouncing
- **Telemetry**: Built-in error tracking and performance monitoring
- **Loading Skeletons**: Better perceived performance with skeleton loaders
- **Clean Architecture**: All styles and scripts extracted to separate files

## Security

- **Content Security Policy (CSP)**: Prevents XSS attacks
- **Local Resources**: All dependencies are served locally (no external CDN calls)
- **Safe Data Binding**: Uses Vue.js `v-text` directives to prevent injection attacks
- **Organized Structure**: Clear separation of HTML, CSS, and JavaScript

## Browser Compatibility

Tested and supported on:
- Chrome/Chromium 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Development

### File Organization

- **HTML**: `index.html` contains only markup, no inline styles or scripts
- **CSS**: All styles in `css/` directory
  - `app.css` contains all custom styles including theme variables
  - Bootstrap and icons are vendor dependencies
- **JavaScript**: All scripts in `js/` directory
  - `app.js` contains the Vue application, telemetry service, and utilities
  - Bootstrap and Vue are vendor dependencies

### Updating Dependencies

To update the local dependencies:

```bash
# Create directories if needed
mkdir -p css js fonts

# Bootstrap CSS
wget -O css/bootstrap.min.css "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"

# Bootstrap Icons CSS
wget -O css/bootstrap-icons.css "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.0/font/bootstrap-icons.css"
# Update font paths in the CSS
sed -i 's|url("./fonts/|url("../fonts/|g' css/bootstrap-icons.css

# Bootstrap Icons Fonts
wget -O fonts/bootstrap-icons.woff2 "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.0/font/fonts/bootstrap-icons.woff2"
wget -O fonts/bootstrap-icons.woff "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.0/font/fonts/bootstrap-icons.woff"

# Bootstrap JavaScript Bundle
wget -O js/bootstrap.bundle.min.js "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"

# Vue.js
wget -O js/vue.global.js "https://unpkg.com/vue@3.3.4/dist/vue.global.js"
```

### Testing Locally

To test the interface locally without the gateway server:

```bash
# Start a simple HTTP server
python3 -m http.server 8080

# Open in browser
# http://localhost:8080/index.html
```

## Production Deployment

The assets are automatically served by the OpenSOVD gateway when accessing the root path `/`. No additional configuration is required.

## Customization

### Theming

Theme variables are defined in `css/app.css`:
- Light theme: Default CSS variables in `:root`
- Dark theme: Overrides in `@media (prefers-color-scheme: dark)`

Key variables:
- `--bg-primary`: Main background color
- `--text-primary`: Main text color
- `--accent-color`: Highlight color (default: #ffcc00)
- `--bg-selected`: Selected item background

### Adding New Features

1. **Styles**: Add new CSS classes to `css/app.css`
2. **JavaScript**: Modify the Vue app in `js/app.js`
3. **HTML**: Update markup in `index.html`

## Telemetry

The interface includes a comprehensive telemetry system (in `js/app.js`) that tracks:
- API call performance and errors
- User interactions (component selection, filtering, etc.)
- Page load and unload events
- JavaScript errors and unhandled promise rejections

Telemetry data is logged to the browser console and can be exported for analysis using `TelemetryService.exportMetrics()`.

## License

These assets are part of the OpenSOVD project and follow the same license terms.