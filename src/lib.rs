use launcher::{
    init_client::InitClient,
    init_server::{Init, InitServer},
};
use launcher::{OpenedRequest, OpenedResponse};
use serde_json::Value as JsonValue;
use std::{convert::TryInto, env, marker::PhantomData, net::Ipv6Addr};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tauri::async_runtime::{channel, Sender};
use tauri::Wry;
use tauri::{
    plugin::{Plugin, Result as PluginResult},
    Manager, Runtime,
};
use tauri::{window::Window, AppHandle};
use tonic::{transport::Server, Request, Response, Status};

use netstat::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use sysinfo::{ProcessExt, System, SystemExt};


pub mod launcher {
    tonic::include_proto!("launcher");
}

pub struct Opener {
    //tx: Sender<String>,
    window: Window
}

#[tonic::async_trait]
impl Init for Opener {
    async fn open(
        &self,
        request: Request<OpenedRequest>,
    ) -> Result<Response<OpenedResponse>, Status> {
        //self.tx.send(format!("{}", request.into_inner().message));
        self.window.emit("opener",format!("{}", request.into_inner().message));
        let reply = launcher::OpenedResponse { accepted: true };
        Ok(Response::new(reply))
    }
}

async fn start_server(window: Window, addr: SocketAddr) {
    let listener = Opener { window: window };
    if let Err(err) = Server::builder()
            .add_service(InitServer::new(listener))
            .serve(addr)
            .await {
        print!("Error {:?}", err);
    };
}

// async fn start_server_ch(tx: Sender<String>, listen_addr: SocketAddr) {
//     let listener = Opener { tx: tx };
//     if let Err(err) = Server::builder()
//         .add_service(InitServer::new(listener))
//         .serve(listen_addr)
//         .await
//     {
//         print!("Error {:?}", err);
//     };
// }
async fn default_broadcaster(
    msg: String,
    scheme: String,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InitClient::connect(format!("{}{}", scheme, addr)).await?;
    let req = tonic::Request::new(OpenedRequest { message: msg });
    let resp = client.open(req).await?;
    println!("{:?}", resp);
    Ok(())
}

fn existing(sys: System, process_pid: usize, process_name: String) -> (bool, SocketAddr) {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let mut port: SocketAddr = SocketAddr::new(IpAddr::from_str("::1").unwrap(), 0);
    let mut found: bool = false;
    for proc in sys
        .process_by_name(&process_name)
        .iter()
        .filter(|proc| proc.pid() != process_pid)
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
    pub window: String,
    pub event: String,
    pub listen: SocketAddr,
    // TODO: pub broadcaster: fn(String,String,SocketAddr) -> Result<(), Box<dyn std::error::Error>>
}

impl Default for HighlanderBuilder {
    fn default() -> Self {
        Self {
            window: "main".to_string(),
            event: "quickening".to_string(),
            listen: SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 0),
        }
    }
}

impl HighlanderBuilder {
    pub fn window(mut self, window: String) -> Self {
        self.window = window;
        self
    }
    pub fn event(mut self, event: String) -> Self {
        self.event = event;
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
            window: self.window,
            event: self.event,
            listen: self.listen,
            phantom: Box::new(|_| ()),
        }
    }
}

pub struct Highlander<R: Runtime> {
    window: String,
    event: String,
    listen: SocketAddr,
    phantom: Box<dyn Fn(PhantomData<R>) + Send + Sync>
    //    broadcaster: fn(String, String, SocketAddr) -> Result<(), Box<dyn std::error::Error>>
}

impl<R: Runtime> Plugin<R> for Highlander<R>{
    fn name(&self) -> &'static str {
        "highlander"
    }

    fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> PluginResult<()> {
        let system: System = System::new_all();
        let current_pid = sysinfo::get_current_pid().unwrap();
        let current_process = system.process(current_pid).unwrap().name().to_string();
        let (found, existing_addr) = existing(system, current_pid, current_process.to_string());
        if !found {
            tauri::async_runtime::spawn(async move {
               start_server(app.get_window(&self.window).unwrap(), self.listen);
            });
        } else {
            tauri::async_runtime::spawn(async move {
                default_broadcaster(
                    env::args()
                        .filter(|arg| !arg.contains(&current_process))
                        .collect::<String>()
                        .to_string(),
                    "http://".to_string(),
                    existing_addr,
                )
            });
        }
        Ok(())
    }
}
