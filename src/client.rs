use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::header::CONTENT_TYPE;
use std::convert::Infallible;

const HTML: &'static str = r#"
<!DOCTYPE html>
<html>
<body>

<h1>Server-Sent Events Demo</h1>
<div id="result"></div>

<script>
var source = new EventSource("http://localhost:3000/events");

source.onmessage = function(event) {
  document.getElementById("result").innerHTML += event.data + "<br>";
};

function pingServer() {
  fetch('http://localhost:3000/ping')
    .then(response => response.text())
    .then(data => console.log(data));
}
</script>

<button onclick="pingServer()">Ping Server</button>

</body>
</html>
"#;

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(HTML))
        .unwrap())
}

#[tokio::main]
async fn main() {
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(handle_request)) }
    });

    let addr = ([127, 0, 0, 1], 3001).into();
    
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
