


pub static HOME_URL: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";

pub struct ImageGrabber {
    request_transmit: Sender<String>,
    response_receive: Receiver<Response>,
}

pub struct GrabberTask {
    request_receive: Receiver<String>,
    response_transmit: Sender<Response>,
}

impl GrabberTask {
    pub fn spawn(request_receive: Receiver<String>, response_transmit: Sender<Response>) {
        std::thread::spawn(|| {
            let mut task = GrabberTask {
                request_receive,
                response_transmit,
            };

            task.run();
        })
    }

    pub async fn run(&self) {
        let client = Client::new();
    }
}

impl ImageGrabber {
    pub fn new() -> ImageGrabber {
        let (request_transmit, request_receive) = flume::bounded(1);
        let (response_transmit, response_receive) = flume::bounded(1);

        GrabberTask::spawn(request_receive, response_transmit);

        ImageGrabber {
            request_transmit,
            response_receive,
        }
    }

    pub fn poll(&self, url: String) -> Result<Response> {
        self.sender.send(url)?;
        self.receiver.recv()?;
    }
}