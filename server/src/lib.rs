pub mod cli;
mod file_service;

use crate::{cli::Cli, file_service::FileServiceImpl};
use anyhow::{anyhow, Result};
use proto::api::file_service_server::FileServiceServer;
use std::{net::{Ipv4Addr, SocketAddr}, path::Path};
use tokio::net::TcpListener;
use tonic::transport::{server::TcpIncoming, Certificate, Identity, Server, ServerTlsConfig};

fn create_tls_config(
    cert_path: &Path,
    key_path: &Path,
    ca_cert_path: &Path,
) -> Result<ServerTlsConfig> {
    let cert = std::fs::read_to_string(cert_path)?;
    let key = std::fs::read_to_string(key_path)?;
    let ca_cert = std::fs::read_to_string(ca_cert_path)?;

    let identity = Identity::from_pem(cert, key);
    let client_ca_cert = Certificate::from_pem(ca_cert);
    let tls_config = ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca_cert);

    Ok(tls_config)
}
use std::net::IpAddr;


pub async fn server_main(args: &Cli) -> Result<()> {
    // let socket_addr = SocketAddr::new(args.address, args.port.unwrap_or(0));
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);
    println!("sockett addr: {:?}", &socket_addr);

    let listener = TcpListener::bind(socket_addr).await?;
    let local_addr = listener.local_addr()?;
    let listener = TcpIncoming::from_listener(listener, true, None).map_err(|e| anyhow!(e))?;

    let file_service_impl = FileServiceImpl::new(args.directory.clone());
    let file_service_server = FileServiceServer::new(file_service_impl);

    let enable_tls =
        args.cert.is_some() && args.key.is_some() && args.ca_cert.is_some() && !args.insecure;

    let mut server = Server::builder();

    if enable_tls {
        let tls_config = create_tls_config(
            args.cert.as_ref().unwrap(),
            args.key.as_ref().unwrap(),
            args.ca_cert.as_ref().unwrap(),
        )?;
        server = server.tls_config(tls_config)?;
    };

    println!("Server address {local_addr}");

    server
    
        .add_service(file_service_server)
        .serve_with_incoming(listener)
        .await?;

    Ok(())
}
