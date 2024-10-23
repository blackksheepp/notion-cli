use rouille::{router, Response, Server};
use std::sync::Once;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use tokio::runtime::Runtime;

use crate::api::auth::authorize;

static INIT: Once = Once::new();
static mut SERVER_HANDLE: Option<ServerHandle> = None;

pub fn initialize_server() {
    INIT.call_once(|| {
        let server_handle = ServerHandle::new();
        unsafe {
            SERVER_HANDLE = Some(server_handle);
        }
    });
}

pub fn start_server() {
    unsafe {
        SERVER_HANDLE
            .as_mut()
            .expect("Server not initialized")
            .start_server();
    }
}

pub fn stop_server() {
    print!("Stopping server...");
    unsafe {
        SERVER_HANDLE
            .as_mut()
            .expect("Server not initialized")
            .stop_server();
    }
}

pub struct ServerHandle {
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ServerHandle {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            thread_handle: None,
        }
    }

    pub fn start_server(&mut self) {
        let running_clone = Arc::clone(&self.running);

        self.thread_handle = Some(thread::spawn(move || {
            let server = Server::new("127.0.0.1:55678", move |request| {
                router!(request,
                    (GET) (/login) => {
                        Response::redirect_302("https://api.notion.com/v1/oauth/authorize?client_id=a13eb1b8-f590-48ef-8c8a-b9d61b0d3fab&response_type=code&owner=user&redirect_uri=http%3A%2F%2Flocalhost%3A55678%2Fredirect")
                    },
                    (GET) (/redirect) => {
                        let code = request.get_param("code").unwrap_or_default();
                        authorize_with_code(&code);
                        Response::text("authorized, return to notion cli.")
                    },
                    _ => Response::empty_404()
                )
            }).expect("Failed to start server");
            loop {
                server.poll();
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
            }
        }));
    }

    pub fn stop_server(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn authorize_with_code(code: &str) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        if let Err(e) = authorize(&code).await {
            eprintln!("Error: {}", e);
        }
    });

    thread::spawn(move || stop_server());
}
