// A Rustls stub for TryTLS
//
// Author: Joachim Viide
// See: https://github.com/HowNetWorks/trytls-rustls-stub
//

use webpki;
use webpki_roots;

use rustls::{ClientConfig, ClientSession, Session, TlsError, RootCertStore, DEFAULT_CIPHERSUITES};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::process;
use std::sync::Arc;

enum Verdict {
    Accept,
    Reject(TlsError),
}

fn parse_args(args: &[String]) -> Result<(String, u16, ClientConfig), Box<dyn Error>> {
    let mut root_store = RootCertStore::empty();
    match args.len() {
        3 => {
            root_store
                .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        }
        4 => {
            let f = File::open(&args[3])?;
            root_store
                .add_parsable_certificates(&rustls_pemfile::certs(&mut BufReader::new(f)).unwrap());
        }
        _ => {
            return Err(From::from("Incorrect number of arguments"));
        }
    };
    let config = ClientConfig::new(
        root_store,
        &[],
        DEFAULT_CIPHERSUITES
    );

    let port = args[2].parse()?;
    Ok((args[1].clone(), port, config))
}

fn communicate(host: String, port: u16, config: ClientConfig) -> Result<Verdict, Box<dyn Error>> {
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(&host).unwrap();
    let rc_config = Arc::new(config);
    let mut client = ClientSession::new(&rc_config, dns_name).unwrap();
    let mut stream = TcpStream::connect((&*host, port))?;

    client.write_all(b"GET / HTTP/1.0\r\nConnection: close\r\nContent-Length: 0\r\n\r\n")?;
    loop {
        while client.wants_write() {
            client.write_tls(&mut stream)?;
        }

        if client.wants_read() {
            if client.read_tls(&mut stream)? == 0 {
                return Err(From::from("Connection closed"));
            }

            if let Err(err) = client.process_new_packets() {
                return match err {
                    TlsError::WebPKIError(..) | TlsError::AlertReceived(_) => {
                        Ok(Verdict::Reject(err))
                    }
                    _ => Err(From::from(format!("{:?}", err))),
                };
            }

            if client.read(&mut [0])? > 0 {
                return Ok(Verdict::Accept);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (host, port, config) = parse_args(&args).unwrap_or_else(|err| {
        println!("Argument error: {}", err);
        process::exit(2);
    });

    match communicate(host, port, config) {
        Ok(Verdict::Accept) => {
            println!("ACCEPT");
            process::exit(0);
        }
        Ok(Verdict::Reject(reason)) => {
            println!("{:?}", reason);
            println!("REJECT");
            process::exit(0);
        }
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    }
}
