use hyper::{
    rt::{self, Future},
    service::service_fn,
    Client, Server,
};
use std::net::SocketAddr;

fn main() {
    let client = Client::new();
    let in_addr = ([127, 0, 0, 1], 8001).into();
    let out_addr: SocketAddr = ([127, 0, 0, 1], 8000).into();
    let server = Server::bind(&in_addr)
        .serve(move || {
            let request_client = client.clone();
            // This is the `Service` that will handle the connection.
            // `service_fn_ok` is a helper to convert a function that
            // returns a Response into a `Service`.
            service_fn(move |mut req| {
                println!("{:?}", req.headers());
                let uri_string = match req.uri().path_and_query().map(|x| x.as_str()) {
                    Some(path) if !path.is_empty() => format!("http://{}{}", out_addr, path),
                    _ => format!("http://{}", out_addr),
                };
                println!("proxying to {}", uri_string);
                let uri = uri_string.parse().unwrap();
                *req.uri_mut() = uri;
                request_client.request(req)
            })
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", in_addr);
    println!("Proxying on http://{}", out_addr);

    rt::run(server);
}
