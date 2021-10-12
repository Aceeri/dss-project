use anyhow::Result;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder};
use tokio::task::{self, JoinHandle};

use std::task::Poll;
use std::collections::{HashMap, HashSet};

pub type HttpResponse = Poll<(String, Result<Bytes>)>;

pub struct HttpGrabber {
    pub request_transmit: Sender<String>,
    pub response_receive: Receiver<HttpResponse>,
    pub response_pool: HashMap<String, Poll<Result<Bytes>>>,
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
                let request = client.get(url.clone());

                // Send Poll::Pending so we don't wait on this in the main thread.
                response_transmit
                    .send_async(Poll::Pending)
                    .await
                    .expect("send pending");

                let result = fetch_image(request).await;
                response_transmit
                    .send_async(Poll::Ready((url, result)))
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
        let (request_transmit, request_receive) = flume::unbounded();
        let (response_transmit, response_receive) = flume::unbounded();

        let join_handle = spawn(request_receive, response_transmit);

        HttpGrabber {
            request_transmit,
            response_receive,
            join_handle,

            response_pool: HashMap::new(),
        }
    }

    // Send a task to fetch a URL.
    pub fn send_request(&mut self, url: String) -> Result<()> {
        //println!("sending request for {:?}", url);
        if !self.response_pool.contains_key(&url) {
            self.request_transmit.send(url.clone())?;
            self.response_pool.insert(url, Poll::Pending);
        }

        Ok(())
    }

    // Check if a response has been received.
    pub async fn poll(&mut self) -> Result<()> {
        let 
        while let Some(url) = self.response_receive.into_stream().await {

        }
        match self.response_receive.recv()? {
            Poll::Pending => {},
            Poll::Ready((url, result)) => {
                self.response_pool.insert(url, Poll::Ready(result));
            }
        }
        Ok(())
    }

    // Grab from responses if it exists.
    pub fn grab_response(&mut self, url: &str) -> Poll<Result<Bytes>> {
        //println!("grabbing: {:?}", url);
        if let Some(result) = self.response_pool.get(url) {
            let result = self.response_pool.remove(url).unwrap();
            result
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn sanity() {}
}
