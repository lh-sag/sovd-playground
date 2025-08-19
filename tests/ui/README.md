# UI Tests for OpenSOVD Gateway

This directory contains UI tests for the OpenSOVD web interface using Playwright and pytest.

## Overview

The UI tests verify the functionality of the OpenSOVD web interface, including:

- Page loading and basic structure
- Component tree display and interaction
- Version selection functionality
- Resource display and expansion
- Dark theme support
- Responsive design
- Error handling
- Performance characteristics

## Prerequisites

1. **Install Dependencies**:

   ```bash
   uv sync
   ```

2. **Install Playwright Browsers**:

   ```bash
   uv run playwright install chromium
   ```

3. **Running Gateway**: The tests require a running OpenSOVD gateway instance.

## Running UI Tests

### Run All UI Tests

```bash
uv run pytest tests/ui/ -m ui
```

### Run Specific Test File

```bash
uv run pytest tests/ui/test_basic_ui.py -v
```

### Run with Browser Visible (for debugging)

```bash
uv run pytest tests/ui/ -m ui --headed
```

### Run with Different Browser

```bash
uv run pytest tests/ui/ -m ui --browser firefox
```

### Run with Video Recording

```bash
uv run pytest tests/ui/ -m ui --video on
```

## Test Structure

### Test Files

- **`test_basic_ui.py`**: Basic smoke tests for core UI functionality
- **`test_opensovd_ui.py`**: Comprehensive UI functionality tests
- **`conftest.py`**: Pytest fixtures and configuration for UI tests

### Test Categories

Tests are organized into several categories:

#### Basic Functionality (`TestOpenSOVDUI`)

- Page loading and rendering
- Sidebar component display
- Server banner functionality
- Component expansion/collapse
- Main panel states
- Component filtering

#### Version Selection (`TestVersionSelection`)

- Version pane display
- Version selection interaction
- API endpoint switching

#### Resource Display (`TestResourceDisplay`)

- Resource icon consistency
- Resource name formatting
- Data/data-list icon matching

#### Performance (`TestUIPerformance`)

- Component loading times
- Hover interaction responsiveness
- Console error detection

## Configuration

### Pytest Markers

- `@pytest.mark.ui`: Marks tests as UI tests
- `@pytest.mark.slow`: Marks tests as slow-running
- `@pytest.mark.integration`: Integration tests

### Playwright Configuration

The tests use the following Playwright settings:

- **Browser**: Chromium (default)
- **Headless**: True (for CI)
- **Viewport**: 1280x720
- **Video**: Retain on failure
- **Screenshots**: Only on failure
- **Tracing**: Retain on failure

## Debugging Tests

### Run Tests with Browser Visible

```bash
uv run pytest tests/ui/test_basic_ui.py --headed --slowmo 1000
```

### Run Single Test with Debug

```bash
uv run pytest tests/ui/test_basic_ui.py::test_page_loads -v --headed --capture=no
```

### View Test Artifacts

After test failures, artifacts are saved to:

- `test-results/`: Screenshots, videos, traces
- View traces at: <https://trace.playwright.dev/>

## Writing New Tests

### Basic Test Structure

```python
@pytest.mark.ui
def test_my_feature(loaded_ui_page: Page):
    """Test description."""
    page = loaded_ui_page

    # Your test code here
    expect(page.locator(".my-element")).to_be_visible()
```

### Available Fixtures

- `page`: Fresh Playwright page
- `loaded_ui_page`: Page with UI already loaded
- `ui_url`: Base URL for the UI
- `wait_for_gateway_ready`: Ensures gateway is responsive
- `mock_components_data`: Mock data for testing
- `mock_version_info`: Mock version information

### Best Practices

1. **Use Explicit Waits**:

   ```python
   page.wait_for_selector(".component-item", timeout=15000)
   ```

2. **Use Playwright's expect()**:

   ```python
   expect(page.locator(".sidebar")).to_be_visible()
   ```

3. **Handle Dynamic Content**:

   ```python
   if component.count() > 0:
       # Test logic for when components exist
   ```

4. **Test Error States**:

   ```python
   # Test what happens when API is unavailable
   ```

## CI Integration

UI tests are integrated into the CI pipeline:

1. **GitHub Actions**: Runs on pull requests and pushes to main
2. **Headless Mode**: Tests run without visible browser
3. **Artifact Collection**: Screenshots and videos saved on failure
4. **Parallel Execution**: Tests run in parallel for speed

### CI Command

```bash
uv run pytest tests/ui/ -m ui --browser chromium --video retain-on-failure
```

## Troubleshooting

### Common Issues

1. **Tests Time Out**:
   - Increase timeout values
   - Check if gateway is running
   - Verify network connectivity

2. **Element Not Found**:
   - Use `wait_for_selector()` before interactions
   - Check element selectors in browser dev tools
   - Verify UI hasn't changed structure

3. **Flaky Tests**:
   - Add appropriate waits for dynamic content
   - Use `page.wait_for_timeout()` sparingly
   - Prefer condition-based waits

### Debug Tips

1. **Add Screenshots**:

   ```python
   page.screenshot(path="debug.png")
   ```

2. **Print Page Content**:

   ```python
   print(page.content())
   ```

3. **Console Logs**:

   ```python
   page.on("console", lambda msg: print(f"Console: {msg.text}"))
   ```

## Test Data

Tests use mock data defined in `conftest.py`:

- Mock component data with various resource types
- Mock version information with multiple API versions
- Configurable test scenarios

## Performance Testing

UI tests include performance benchmarks:

- Component loading under 15 seconds
- Hover interactions under 1 second
- No console errors during normal usage

## Accessibility Testing

Basic accessibility checks are included:

- Proper heading structure
- Form label associations
- Button focusability
- Keyboard navigation (future enhancement)

## Future Enhancements

Planned improvements:

- Visual regression testing
- Cross-browser testing (Firefox, Safari)
- Mobile-specific UI tests
- Accessibility compliance testing
- Performance monitoring integration
