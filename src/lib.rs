use anyhow::Result;
use futures::future::join_all;
use pyo3::prelude::*;
use reqwest::{Client, ClientBuilder, StatusCode};
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Debug)]
#[pyclass]
struct ScrapingError {
    message: String,
}

impl std::fmt::Display for ScrapingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ScrapingError {}

impl From<reqwest::Error> for ScrapingError {
    fn from(err: reqwest::Error) -> Self {
        ScrapingError {
            message: err.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for ScrapingError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ScrapingError {
            message: err.to_string(),
        }
    }
}

/// A fast web scraper implemented in Rust
#[pyclass]
struct FastScraper {
    client: Client,
    max_retries: u32,
    rate_limit: Option<Arc<Semaphore>>,
}

#[pymethods]
impl FastScraper {
    #[new]
    #[pyo3(signature = (
        timeout_ms=5000,
        max_retries=3,
        max_concurrent_requests=None
    ))]
    fn new(
        timeout_ms: u64,
        max_retries: u32,
        max_concurrent_requests: Option<usize>,
    ) -> PyResult<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let rate_limit = max_concurrent_requests.map(|n| Arc::new(Semaphore::new(n)));

        Ok(FastScraper {
            client,
            max_retries,
            rate_limit,
        })
    }

    /// Fetch a URL and return the HTML content with retry mechanism
    fn fetch(&self, url: &str) -> PyResult<String> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut retries = 0;

        while retries < self.max_retries {
            let result = runtime.block_on(async {
                if let Some(rate_limit) = &self.rate_limit {
                    let _permit = rate_limit.acquire().await.unwrap();
                }

                match self.client.get(url).send().await {
                    Ok(response) => {
                        let status = response.status();
                        if status.is_success() {
                            Ok(response.text().await?)
                        } else if status.is_server_error() && retries < self.max_retries - 1 {
                            Err(ScrapingError {
                                message: format!("HTTP error: {}", status),
                            })
                        } else {
                            Err(ScrapingError {
                                message: format!("HTTP error: {}", status),
                            })
                        }
                    }
                    Err(e) => Err(ScrapingError {
                        message: e.to_string(),
                    }),
                }
            });

            match result {
                Ok(content) => return Ok(content),
                Err(e) => {
                    retries += 1;
                    if retries == self.max_retries {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                            e.to_string(),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(1000 * retries as u64));
                }
            }
        }
        unreachable!()
    }

    /// Fetch multiple URLs concurrently
    fn fetch_many(&self, urls: Vec<String>) -> PyResult<Vec<String>> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = self.client.clone();
        let rate_limit = self.rate_limit.clone();

        let results = runtime.block_on(async {
            let mut futures = Vec::new();
            for url in urls {
                let client = &client;
                let rate_limit = rate_limit.clone();

                let future = async move {
                    if let Some(rate_limit) = rate_limit {
                        let _permit = rate_limit.acquire().await.unwrap();
                    }

                    match client.get(&url).send().await {
                        Ok(response) => match response.status() {
                            StatusCode::OK => Ok(response.text().await?),
                            status => Err(ScrapingError {
                                message: format!("HTTP error: {}", status),
                            }),
                        },
                        Err(e) => Err(ScrapingError {
                            message: e.to_string(),
                        }),
                    }
                };
                futures.push(future);
            }

            join_all(futures).await
        });

        results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Extract elements using CSS selector
    fn select(&self, html: &str, selector: &str) -> PyResult<Vec<String>> {
        let document = Html::parse_document(html);
        let selector = match Selector::parse(selector) {
            Ok(s) => s,
            Err(e) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid selector: {}",
                    e
                )))
            }
        };

        let elements: Vec<String> = document
            .select(&selector)
            .map(|element| element.text().collect::<Vec<_>>().join(""))
            .collect();

        Ok(elements)
    }

    /// Extract attributes from elements using CSS selector
    fn select_attr(&self, html: &str, selector: &str, attr: &str) -> PyResult<Vec<String>> {
        let document = Html::parse_document(html);
        let selector = match Selector::parse(selector) {
            Ok(s) => s,
            Err(e) => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid selector: {}",
                    e
                )))
            }
        };

        let elements: Vec<String> = document
            .select(&selector)
            .filter_map(|element| element.value().attr(attr).map(String::from))
            .collect();

        Ok(elements)
    }

    /// Fetch and parse JSON from a URL
    fn fetch_json(&self, url: &str) -> PyResult<PyObject> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let response = runtime.block_on(async {
            if let Some(rate_limit) = &self.rate_limit {
                let _permit = rate_limit.acquire().await.unwrap();
            }

            self.client
                .get(url)
                .send()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .json::<serde_json::Value>()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })?;

        Python::with_gil(|py| {
            let json_str = response.to_string();
            let json_module = py.import("json")?;
            let json_dict = json_module.call_method1("loads", (json_str,))?;
            Ok(json_dict.into())
        })
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FastScraper>()?;
    Ok(())
}
