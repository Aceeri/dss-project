use anyhow::Result;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use reqwest::header::OccupiedEntry;
use reqwest::{Client, RequestBuilder};
use tokio::task::{self, JoinHandle};

use std::task::Poll;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};

pub type HttpResponse = Poll<Result<Bytes>>;

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
        
        let responses: Arc<Mutex<HashMap<String, HttpResponse>>> = Arc::new(Mutex::new(HashMap::new()));

        while let Some(url) = requests.next().await {
            let client = client.clone(); // not sure if we really need to clone this?
            let response_transmit = response_transmit.clone();

            task::spawn(async move {
                let mut locked_responses = responses.lock().expect("get response lock");
                match locked_responses.entry(url.clone()) {
                    Entry::Occupied(entry) => {
                        // Maybe we should do some error handling here in case of bad requests?
                        let response = match entry.get() {
                            // this might be kind of rough, might be copying a lot of data.
                            Poll::Ready(result) => Poll::Ready(*result),
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
                        responses.lock().expect("get response lock for insert").insert(url, Poll::Ready(response));
                    }
                }
            });
        }
    })
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

        let join_handle = spawn(request_receive, response_transmit);

        HttpGrabber {
            request_transmit,
            response_receive,
            join_handle,

            response_pool: HashMap::new(),
        }
    }

    // Alright, sending individual tasks is kind of complicated, so let's just send 1 request and check for the response before moving on.
    pub fn poll_request(&mut self, url: String) -> Result<HttpResponse> {
        println!("polling request for {:?}", url);
        self.request_transmit.send(url.clone())?;
        let response = self.response_receive.recv()?;
        Ok(response)
    }

    // Grab from responses if it exists and is ready
    pub fn grab_response(&mut self, url: &str) -> Poll<Result<Bytes>> {
        //println!("grabbing: {:?}", url);
        if let Some(Poll::Ready(_)) = self.response_pool.get(url) {
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
