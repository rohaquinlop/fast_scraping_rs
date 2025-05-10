# Fast Scraping RS

A high-performance web scraping library implemented in Rust with Python bindings. This library provides fast and efficient web scraping capabilities with features like concurrent requests, rate limiting, and CSS selector support.

## Features

- Fast HTML fetching with retry mechanism
- Concurrent request handling with rate limiting
- CSS selector support for element extraction
- JSON response parsing
- Configurable timeouts and retries
- Error handling with detailed messages

## Installation

```bash
pip install fast-scraping-rs
```

## Usage

```python
from fast_scraping_rs import FastScraper

# Create a scraper instance with custom configuration
scraper = FastScraper(
    timeout_ms=5000,  # 5 second timeout
    max_retries=3,    # Retry failed requests up to 3 times
    max_concurrent_requests=5  # Allow up to 5 concurrent requests
)

# Fetch a single page
html = scraper.fetch("https://example.com")

# Extract elements using CSS selectors
titles = scraper.select(html, "h1")
links = scraper.select_attr(html, "a", "href")

# Fetch multiple pages concurrently
urls = [
    "https://example.com/page1",
    "https://example.com/page2",
    "https://example.com/page3"
]
results = scraper.fetch_many(urls)

# Fetch and parse JSON
json_data = scraper.fetch_json("https://api.example.com/data")
```

## API Reference

### FastScraper

The main scraper class that provides all scraping functionality.

#### Constructor

```python
FastScraper(
    timeout_ms: int = 5000,
    max_retries: int = 3,
    max_concurrent_requests: Optional[int] = None
)
```

- `timeout_ms`: Request timeout in milliseconds
- `max_retries`: Maximum number of retries for failed requests
- `max_concurrent_requests`: Maximum number of concurrent requests (None for no limit)

#### Methods

- `fetch(url: str) -> str`: Fetch a single URL and return the HTML content
- `fetch_many(urls: List[str]) -> List[str]`: Fetch multiple URLs concurrently
- `select(html: str, selector: str) -> List[str]`: Extract elements using CSS selector
- `select_attr(html: str, selector: str, attr: str) -> List[str]`: Extract attributes from elements
- `fetch_json(url: str) -> Any`: Fetch and parse JSON from a URL

## Error Handling

The library provides detailed error messages for various failure scenarios:

- Network errors
- Timeout errors
- Invalid CSS selectors
- HTTP errors
- JSON parsing errors

## Performance

The library is optimized for performance with:

- Concurrent request handling
- Efficient HTML parsing
- Minimal memory usage
- Configurable rate limiting

## Development

### Prerequisites

- Rust (latest stable)
- Python 3.8+
- maturin

### Building

```bash
# Install maturin
pip install maturin

# Build the project
maturin develop
```

### Running Tests

```bash
pytest tests/
```

## License

MIT License