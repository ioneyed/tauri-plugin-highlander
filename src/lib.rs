use launcher::{
    init_client::InitClient,
    init_server::{Init, InitServer},
    OpenedRequest, OpenedResponse,
};
use serde_json::Value as JsonValue;
use std::{
    env,
    marker::PhantomData,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    process::exit,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tauri::{
    plugin::{Plugin, Result as PluginResult},
    AppHandle, Manager, Runtime,
};
use tonic::{transport::Server, Request, Response, Status};

use netstat::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use sysinfo::{ProcessExt, System, SystemExt};

pub mod launcher {
    tonic::include_proto!("launcher");
}

pub struct Opener<R: Runtime> {
    event: String,
    handle: Arc<Mutex<AppHandle<R>>>,
    label: String,
}

#[tonic::async_trait]
impl<R: Runtime> Init for Opener<R> {
    async fn open(
        &self,
        request: Request<OpenedRequest>,
    ) -> Result<Response<OpenedResponse>, Status> {
        let req_message = request.into_inner().message;
        match self.handle.clone() {
            app_handle => {
                let _ah = app_handle.lock().unwrap();
                if let Some(_window) = _ah.get_window(&self.label) {
                    let _emitted = _window.emit(&self.event, format!("{}", req_message));
                }
            }
        }
        println!("{}", req_message);
        let reply = launcher::OpenedResponse { accepted: true };
        Ok(Response::new(reply))
    }
}

async fn start_server<R: Runtime>(
    handle: Arc<Mutex<AppHandle<R>>>,
    event: String,
    label: String,
    listen: SocketAddr,
) {
    let listener = Opener {
        event,
        handle,
        label,
    };
    if let Err(err) = Server::builder()
        .add_service(InitServer::new(listener))
        .serve(listen)
        .await
    {
        println!("Error {:?}", err);
    };
}

async fn default_broadcaster(
    msg: String,
    scheme: String,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InitClient::connect(format!("{}{}", scheme, addr)).await?;
    let req = tonic::Request::new(OpenedRequest { message: msg });
    let _resp = client.open(req).await?;
    Ok(())
}

fn existing() -> (bool, SocketAddr) {
    let system: System = System::new_all();
    let current_pid = match sysinfo::get_current_pid() {
        Ok(_pid) => _pid,
        Err(_) => 0,
    };
    let current_process = system.process(current_pid).unwrap().name().to_string();
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let mut port: SocketAddr = SocketAddr::new(IpAddr::from_str("::1").unwrap(), 0);
    let mut found: bool = false;
    for proc in system
        .process_by_name(&current_process)
        .iter()
        .filter(|proc| proc.pid() != current_pid)
    {
        for si in get_sockets_info(af_flags, proto_flags).unwrap() {
            if si
                .associated_pids
                .iter()
                .any(|spid| -> bool { *spid == proc.pid() as u32 })
            {
                found = true;
                match si.protocol_socket_info {
                    ProtocolSocketInfo::Tcp(_sitcp) => {
                        port = SocketAddr::new(_sitcp.local_addr, _sitcp.local_port);
                    }
                    ProtocolSocketInfo::Udp(_siudp) => {
                        port = SocketAddr::new(_siudp.local_addr, _siudp.local_port);
                    }
                }
            }
        }
    }
    (found, port)
}

pub struct HighlanderBuilder {
    pub label: String,
    pub event: String,
    pub listen: SocketAddr,
    // TODO: pub broadcaster: fn(String,String,SocketAddr) -> Result<(), Box<dyn std::error::Error>>
}

impl Default for HighlanderBuilder {
    fn default() -> Self {
        Self {
            label: "main".to_string(),
            event: "quickening".to_string(),
            listen: SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 0),
        }
    }
}

impl HighlanderBuilder {
    pub fn new(event: String, label: String, listen: SocketAddr) -> Self{
        Self{
            event,
            label,
            listen
        }
    }
    pub fn event(mut self, event: String) -> Self {
        self.event = event;
        self
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = label;
        self
    }
    
    pub fn listen(mut self, listen: SocketAddr) -> Self {
        self.listen = listen;
        self
    }

    // pub fn broadcaster(mut self, f: fn(String, String, SocketAddr) -> Result<(), Box<dyn std::error::Error>>) -> Self {
    //     self.broadcaster = f;
    //     self
    // }

    pub fn build<R: Runtime>(self) -> Highlander<R> {
        Highlander {
            label: self.label,
            event: self.event,
            listen: self.listen,
            _phantom: Box::new(|_| ()),
        }
    }
}

pub struct Highlander<R: Runtime> {
    label: String,
    event: String,
    listen: SocketAddr,
    #[warn(dead_code)]
    _phantom: Box<dyn Fn(PhantomData<R>) + Send + Sync>, //    broadcaster: fn(String, String, SocketAddr) -> Result<(), Box<dyn std::error::Error>>
}

impl<R: Runtime> Plugin<R> for Highlander<R> {
    fn name(&self) -> &'static str {
        "highlander"
    }

    fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> PluginResult<()> {
        let (found, existing_addr) = existing();
        let current_window = Arc::new(Mutex::new(app.clone()));
        if !found {
            let _event = self.event.clone();
            let _label = self.label.clone();
            let _listen = self.listen.clone();
            tauri::async_runtime::spawn(async move {
                start_server( current_window, _event, _label, _listen).await;
            });
        } else {
            tauri::async_runtime::block_on(async move {
                let _broadcasted = default_broadcaster(
                    env::args()
                        .collect::<Vec<String>>()
                        .join(" "),
                    "http://".to_string(),
                    existing_addr,
                )
                .await;
            });
            exit(1);
        }
        Ok(())
    }
}
