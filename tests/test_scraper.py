import pytest
from fast_scraping_rs import FastScraper


@pytest.fixture
def scraper():
    return FastScraper(
        timeout_ms=5000, max_retries=2, max_concurrent_requests=5
    )


def test_fetch_simple_page(scraper):
    """Test fetching a simple webpage."""
    html = scraper.fetch("https://httpbin.org/html")
    assert html is not None
    assert "<html>" in html.lower()
    assert "<body>" in html.lower()


def test_fetch_with_retry(scraper):
    """Test retry mechanism with a temporarily failing URL."""
    # This URL will fail on first attempt but succeed on retry
    with pytest.raises(Exception) as exc_info:
        scraper.fetch("https://httpbin.org/status/503")
    assert "HTTP error: 503" in str(exc_info.value)


def test_fetch_many(scraper):
    """Test concurrent fetching of multiple URLs."""
    urls = [
        "https://httpbin.org/html",
        "https://httpbin.org/status/200",
        "https://httpbin.org/status/200",
    ]
    results = scraper.fetch_many(urls)
    assert len(results) == 3
    assert all(result is not None for result in results)


def test_select_elements(scraper):
    """Test CSS selector functionality."""
    html = """
    <html>
        <body>
            <h1>Test Title</h1>
            <p>Test paragraph</p>
            <h1>Another Title</h1>
        </body>
    </html>
    """
    titles = scraper.select(html, "h1")
    assert len(titles) == 2
    assert "Test Title" in titles
    assert "Another Title" in titles


def test_select_attributes(scraper):
    """Test attribute selection functionality."""
    html = """
    <html>
        <body>
            <a href="https://example.com">Link 1</a>
            <a href="https://test.com">Link 2</a>
        </body>
    </html>
    """
    links = scraper.select_attr(html, "a", "href")
    assert len(links) == 2
    assert "https://example.com" in links
    assert "https://test.com" in links


def test_fetch_json(scraper):
    """Test JSON fetching and parsing."""
    json_data = scraper.fetch_json("https://httpbin.org/json")
    assert json_data is not None
    assert "slideshow" in json_data


def test_rate_limiting(scraper):
    """Test rate limiting functionality."""
    # Create a scraper with a very low concurrency limit
    limited_scraper = FastScraper(
        timeout_ms=5000,
        max_retries=2,
        max_concurrent_requests=2,  # Only allow 2 concurrent requests
    )

    # Use a URL that will take some time to respond
    urls = ["https://httpbin.org/delay/1"] * 4
    results = limited_scraper.fetch_many(urls)

    # Verify all requests completed successfully
    assert len(results) == 4
    assert all(result is not None for result in results)


def test_error_handling(scraper):
    """Test error handling for invalid URLs."""
    with pytest.raises(Exception):
        scraper.fetch("https://invalid-url-that-does-not-exist.com")


def test_timeout_handling(scraper):
    """Test timeout handling."""
    with pytest.raises(Exception):
        # This URL will delay for 10 seconds, which should exceed our 5-second timeout
        scraper.fetch("https://httpbin.org/delay/10")


def test_invalid_selector(scraper):
    """Test handling of invalid CSS selectors."""
    html = "<div>Test</div>"
    try:
        scraper.select(html, "[[[")  # Clearly invalid CSS selector
        pytest.fail("Expected ValueError but no exception was raised")
    except Exception as e:
        assert "Invalid selector" in str(e)


def test_empty_urls(scraper):
    """Test handling of empty URL list."""
    results = scraper.fetch_many([])
    assert len(results) == 0
