use anyhow::Result;
use std::net::TcpStream;
use std::time::Instant;
use url::Url;

///
/// 测试网络连接
///
/// 返回被测试url，以及测试时间 ms
pub(crate) fn test_connection(url: String) -> Result<(String, u128)> {
    let url_paser = Url::parse(&url)?;
    let host = match url_paser.host_str() {
        Some(host) => host,
        None => panic!("Invalid URL: {}", url_paser),
    };

    let port = url_paser.port_or_known_default().unwrap_or(80);

    // 解析域名并连接到服务器
    let address = format!("{}:{}", host, port);
    let start = Instant::now();
    let _ = TcpStream::connect(address.clone())?;
    let duration = start.elapsed();

    Ok((url, duration.as_millis()))
}
