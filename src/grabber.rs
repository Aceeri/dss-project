use anyhow::Result;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use reqwest::header::OccupiedEntry;
use reqwest::{Client, RequestBuilder};
use tokio::task::{self, JoinHandle};
use tokio::sync::Mutex;

use std::task::Poll;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::sync::{Arc};

pub type HttpResponse = Poll<Result<Bytes>>;
type ResponseCache = Arc<Mutex<HashMap<String, HttpResponse>>>;

// Mostly just following https://tokio.rs/tokio/tutorial/shared-state for context.

pub struct HttpGrabber {
    pub request_transmit: Sender<String>,
    pub response_receive: Receiver<HttpResponse>,
}

pub async fn spawn(
    request_receive: Receiver<String>,
    response_transmit: Sender<HttpResponse>,
) {
    let client = Client::new();
    let mut requests = request_receive.into_stream();
    
    let responses: ResponseCache = Arc::new(Mutex::new(HashMap::new()));

    while let Some(url) = requests.next().await {
        let client = client.clone();
        let responses = responses.clone();
        let response_transmit = response_transmit.clone();

        task::spawn(async move {
            process(client, responses, url, response_transmit).await;
        });
    }
}

pub async fn process(client: reqwest::Client, cache: ResponseCache, url: String, response_transmit: Sender<HttpResponse>) {
    let mut locked_responses = cache.lock().await;

    match locked_responses.entry(url.clone()) {
        Entry::Occupied(entry) => {
            let response = match entry.get() {
                Poll::Ready(Ok(bytes)) => Poll::Ready(Ok(bytes.clone())), // Bytes is reference counted so cloning isn't too heavy.
                Poll::Ready(Err(_)) => Poll::Pending,
                Poll::Pending => Poll::Pending,
            };

            response_transmit.send_async(response).await.expect("send response")
        }
        Entry::Vacant(entry) => {
            entry.insert(Poll::Pending);

            // need to kill the mutex early, otherwise this is going to be locked forever.
            drop(locked_responses); 
            response_transmit.send_async(Poll::Pending).await.expect("send pending");

            let request = client.get(url.clone());
            let response = fetch_http(request).await;
            cache.lock().await.insert(url, Poll::Ready(response));
        }
    }
}

pub async fn fetch_http(request: RequestBuilder) -> Result<Bytes> {
    let response = request.send().await?;
    let bytes = response.bytes().await?;
    Ok(bytes)
}

impl HttpGrabber {
    pub fn new() -> HttpGrabber {
        let (request_transmit, request_receive) = flume::bounded(1);
        let (response_transmit, response_receive) = flume::bounded(1);

        task::spawn(async move {
            spawn(request_receive, response_transmit).await;
        });

        HttpGrabber {
            request_transmit,
            response_receive,
        }
    }

    // Alright, sending individual tasks is kind of complicated, so let's just send 1 request and check for the response before moving on.
    pub fn poll_request(&mut self, url: String) -> Result<HttpResponse> {
        //println!("polling request for {:?}", url);
        self.request_transmit.send(url.clone())?;
        let response = self.response_receive.recv()?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn sanity() {}
}