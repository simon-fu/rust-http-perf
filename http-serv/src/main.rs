use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Client};
use anyhow::{Result, Context};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type HttpClient = Client<hyper::client::HttpConnector>;


// async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     use hyper::{Method, StatusCode};
//     match (req.method(), req.uri().path()) {
//         // Serve some instructions at /
//         (&Method::GET, "/") => Ok(Response::new(Body::from(
//             "NOW: Friday, 07-Jan-2022 07:00:51 GMT",
//         ))),


//         // Return the 404 Not Found for other routes.
//         _ => {
//             let mut not_found = Response::default();
//             *not_found.status_mut() = StatusCode::NOT_FOUND;
//             Ok(not_found)
//         }
//     }
// }


async fn proxy(client: HttpClient, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    client.request(req).await
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = ([127, 0, 0, 1], 5000).into();
    let url_str = "http://127.0.0.1/checked-out";
    // let url_str = "http://www.baidu.com/";

    let url = url_str.parse::<hyper::Uri>().with_context(||"fail to parsed url")?;
    // {
    //     let client = Client::new();
    //     let mut res = client.get(url.clone()).await?;
    //     while let Some(next) = hyper::body::HttpBody::data(&mut res).await {
    //         let chunk = next?;
    //         tokio::io::AsyncWriteExt::write_all(&mut tokio::io::stdout(), &chunk).await?;
    //     }
    
    //     println!("\n\nDone!");
    // }

    // let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(echo)) });

    let service = make_service_fn(move |_conn: &AddrStream| {
        let client = Client::new();
        let url0 = url.clone();
        async move {
            Ok::<_, GenericError>(service_fn(move |mut req| {
                // println!("{:?}", req);
                *req.uri_mut() = url0.clone();
                if let Some(host) = req.headers_mut().get_mut("Host") {
                    let value = hyper::header::HeaderValue::from_str(url0.host().unwrap()).unwrap();
                    *host = value;
                }
                proxy(client.clone(), req)
            }))
        }
    });
    

    let server = Server::bind(&addr).serve(service);

    println!("proxy to [{}], threads = 4", url_str, );
    println!("Listening on [http://{}]", addr);

    server.await?;

    Ok(())
}
