use anyhow::Result;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder};
use tokio::task::{self, JoinHandle};

use std::task::Poll;

pub type HttpResponse = Poll<Result<Bytes>>;

pub struct HttpGrabber {
    pub request_transmit: Sender<String>,
    pub response_receive: Receiver<HttpResponse>,
    join_handle: JoinHandle<()>,
}

pub fn spawn(
    request_receive: Receiver<String>,
    response_transmit: Sender<HttpResponse>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let client = Client::new();
        let mut requests = request_receive.into_stream();

        while let Some(url) = requests.next().await {
            let client = client.clone();
            let response_transmit = response_transmit.clone();
            task::spawn(async move {
                let request = client.get(url);
                response_transmit
                    .send_async(Poll::Pending)
                    .await
                    .expect("send pending");

                let result = fetch_image(request).await;
                response_transmit
                    .send_async(Poll::Ready(result))
                    .await
                    .expect("send result");
            });
        }
    })
}

pub async fn fetch_image(request: RequestBuilder) -> Result<Bytes> {
    let response = request.send().await?;
    let bytes = response.bytes().await?;
    Ok(bytes)
}

impl HttpGrabber {
    pub fn new() -> HttpGrabber {
        let (request_transmit, request_receive) = flume::bounded(1);
        let (response_transmit, response_receive) = flume::bounded(1);

        let join_handle = spawn(request_receive, response_transmit);

        HttpGrabber {
            request_transmit,
            response_receive,
            join_handle,
        }
    }

    pub fn poll(&self, url: String) -> HttpResponse {
        self.request_transmit.send(url)?;
        let response = self.response_receive.recv()?;
        response
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn sanity() {}
}
