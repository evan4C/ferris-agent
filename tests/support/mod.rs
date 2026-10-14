use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct CapturedRequest {
    pub headers: String,
    pub body: Vec<u8>,
}

pub async fn spawn_http_response(
    status: &str,
    content_type: &str,
    chunks: Vec<Vec<u8>>,
) -> (String, JoinHandle<CapturedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let status = status.to_owned();
    let content_type = content_type.to_owned();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        let (header_end, content_length) = loop {
            let count = socket.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "client closed before sending a full request");
            request.extend_from_slice(&buffer[..count]);

            if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if request.len() >= header_end + 4 + content_length {
                    break (header_end, content_length);
                }
            }
        };

        let headers = String::from_utf8_lossy(&request[..header_end]).into_owned();
        let body_start = header_end + 4;
        let body = request[body_start..body_start + content_length].to_vec();
        let response_body_len: usize = chunks.iter().map(Vec::len).sum();
        let response_headers = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {response_body_len}\r\nConnection: close\r\n\r\n"
        );
        socket.write_all(response_headers.as_bytes()).await.unwrap();
        for chunk in chunks {
            socket.write_all(&chunk).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        CapturedRequest { headers, body }
    });
    (format!("http://{address}/chat/completions"), task)
}
