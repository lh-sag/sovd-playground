# Integration Tests

Integration tests for the sovd-gateway server using pytest.

## Setup

```bash
# Install dependencies
uv sync
```

## Running Tests

```bash
# List all tests
uv run pytest --collect-only

# Run all tests
uv run pytest

# Run with verbose output
uv run pytest -v -s

# Run specific test
uv run pytest -k "test_name"

# Use pre-built binary
uv run pytest --sovd-gateway-bin=./target/release/sovd-gateway
```

## Test Options

- `--sovd-gateway-bin`: Path to sovd-gateway binary (defaults to cargo run)
- `--sovd-gateway-args`: Additional arguments for the gateway
- `--sovd-gateway-profile`: Cargo profile for building (default: release)

## Fixtures

- **`gateway`**: Automatically starts and stops the gateway
- **`gateway_manager`**: Manual gateway lifecycle control
- **`gateway_url`**: Base URL of the running gateway
- **`client`**: httpx async client configured with gateway URL

## Example Test

```python
async def test_components(client):
    """Test fetching components."""
    response = await client.get("/v1/components")
    assert response.status_code == 200
```
