use futures::future::{self, Either};
use hyper::{
    rt::{self, Future},
    service::service_fn,
    Body, Client, HeaderMap, Response, Server,
};
use std::net::SocketAddr;

fn operation(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-amz-target")
        .and_then(|target| target.to_str().ok().and_then(|s| s.splitn(2, '.').last()))
}

// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html
fn throughput_exceeded_exceeded() -> Response<Body> {
    Response::builder()
        .status(400)
        .header(
            "content-type", " application/x-amz-json-1.0"
        )
        .body(
            r#"{"__type":"com.amazonaws.dynamodb.v20120810#ProvisionedThroughputExceededException",
    "message":"You exceeded your maximum allowed provisioned throughput for a table or for one or more global secondary indexes. To view performance metrics for provisioned throughput vs. consumed throughput, open the Amazon CloudWatch console."}"#.into()
        )
        .unwrap()
}

fn fail(_operation: &str) -> Response<Body> {
    // todo only return a possible error
    throughput_exceeded_exceeded()
}

fn main() {
    let client = Client::new();
    let in_addr = ([127, 0, 0, 1], 8001).into();
    let out_addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

    let server = Server::bind(&in_addr)
        .serve(move || {
            let request_client = client.clone();
            service_fn(move |mut req| {
                if let Some(operation) = operation(req.headers()) {
                    if rand::random::<f64>() > 0.5 {
                        println!("failing {} operation", operation);
                        return Either::A(future::ok(fail(operation)));
                    }
                }
                let uri_string = match req.uri().path_and_query().map(|x| x.as_str()) {
                    Some(path) if !path.is_empty() => format!("http://{}{}", out_addr, path),
                    _ => format!("http://{}", out_addr),
                };
                println!("proxying to {}", uri_string);
                let uri = uri_string.parse().unwrap();
                *req.uri_mut() = uri;
                Either::B(request_client.request(req))
            })
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", in_addr);
    println!("Proxying on http://{}", out_addr);

    rt::run(server);
}
