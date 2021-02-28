mod config;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let mut time = timestamp(data.epoch);
    let mut state = data.state.lock().unwrap();
    let sequence = state.sequence + 1;
    if sequence > config::MAX_SEQUENCE {
        panic!("max sequence was too short")
    }

    // Accept clocks going backwards for up to 1 second
    // This will skew up to 1000x ids in the backwards ms
    // TODO(intern): skew up to 2x distributed over the next 1000ms
    if time < state.time {
        if time + 1000 < state.time {
            panic!("time-travelling {}ms", state.time - time)
        }

        time = state.time;
    }

    if time == state.time {
        state.sequence = sequence;
    } else {
        state.time = time;
        state.sequence = 0;
    }

    let id: u64 =
        (time << config::TIMESTAMP_SHIFT) + (sequence << config::SEQUENCE_SHIFT) + data.node_suffix;
    HttpResponse::Ok().body(id.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let epoch: SystemTime = UNIX_EPOCH + std::time::Duration::from_millis(config::DRLC);

    let region: u64 = 2;
    let node: u64 = 5;
    let suffix: u64 = (region << config::NODE_BITS) + node;

    let data = web::Data::new(RuidGeneratorData {
        epoch: epoch,
        node_suffix: suffix,
        state: Mutex::new(RuidGeneratorState {
            time: 0,
            sequence: 0,
        }),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::get().to(id_endpoint))
    })
    .bind("127.0.0.1:8080")?
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
