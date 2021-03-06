#[macro_use]
mod config;

use actix_web::{client::Client, web, App, HttpResponse, HttpServer, Responder};
use std::env;
use std::net::Ipv4Addr;
use std::str;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct RuidGeneratorState {
    time: u64,
    sequence: u64,
}

struct RuidGeneratorData {
    epoch: SystemTime,
    node_suffix: u64,
    state: Mutex<RuidGeneratorState>,
}

async fn id_endpoint(data: web::Data<RuidGeneratorData>) -> impl Responder {
    let sequence: u64;
    let mut time = timestamp(data.epoch);
    let mut err: String = String::from("");
    {
        let mut state = data.state.lock().unwrap();
        sequence = state.sequence + 1;
        if sequence > config::MAX_SEQUENCE {
            err = String::from("max sequence was too short");
        }

        // Accept clocks going backwards for up to 1 second
        // This will skew up to MMTTT ids in the backwards ms
        // TODO(intern): skew up to 2x distributed over the next MMTTT.
        if time < state.time {
            if time + config::MMTTT > state.time {
                err = format!("time-travelling {}ms", state.time - time)
            } else {
                time = state.time;
            }
        }

        if err == "" {
            if time == state.time {
                state.sequence = sequence;
            } else {
                state.time = time;
                state.sequence = 0;
            }
        }
    }

    // All errors self-resolve after a while, so panic outside the mutex lock.
    if err != "" {
        panic!(err);
    }

    let id: u64 =
        (time << config::TIMESTAMP_SHIFT) + (sequence << config::SEQUENCE_SHIFT) + data.node_suffix;
    HttpResponse::Ok().body(id.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Specify exactly 1 arg with cluster id");
    }
    let cluster: u64 = args[1].parse::<u64>().expect("cluster id not u64");
    if cluster > config::MAX_CLUSTER {
        panic!("Cluster gt {}", config::MAX_CLUSTER);
    }
    let node: u64 = get_node_id().await;
    let suffix: u64 = (cluster << config::NODE_BITS) + node;

    let epoch: SystemTime = UNIX_EPOCH + Duration::from_millis(config::DRLC);
    let t: u64 = timestamp(epoch);
    thread::sleep(Duration::from_millis(config::MMTTT));
    let initial_time: u64 = timestamp(epoch);
    if t + config::MMTTT > initial_time {
        panic!("Time travelled while time travelling")
    }

    let data = web::Data::new(RuidGeneratorData {
        epoch: epoch,
        node_suffix: suffix,
        state: Mutex::new(RuidGeneratorState {
            time: initial_time,
            sequence: 0,
        }),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::get().to(id_endpoint))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[inline(always)]
fn timestamp(epoch: SystemTime) -> u64 {
    let t: u128 = SystemTime::now()
        .duration_since(epoch)
        .expect("time-travelling before DRLC")
        .as_millis();
    if t > config::MAX_TIMESTAMP {
        panic!("ruid is not future-proof enough");
    }

    t as u64
}

async fn get_node_id() -> u64 {
    let ip_utf8_bytes = Client::default()
        .get(node_ip_uri!())
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    let mut ip_string = String::from(str::from_utf8(&ip_utf8_bytes).unwrap());
    ip_string.retain(|c| !c.is_whitespace());

    let ip = ip_string.parse::<Ipv4Addr>().unwrap();

    if config::NODE_BITS > 8 {
        panic!("Can only parse up to 8 bits for node")
    }
    let last_octet = ip.octets()[3] as u64;
    let mask = (1 << config::NODE_BITS) - 1;

    last_octet & mask
}
