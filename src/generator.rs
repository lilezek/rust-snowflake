use lazy_static::lazy_static;
use std::sync::atomic::{AtomicI32, AtomicU64};

use crate::globals;

static GENERATOR_COUNT: AtomicI32 = AtomicI32::new(-1);
static GENERATOR_MILLIS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref MACHINE_ID: u16 = std::env::var("MACHINE_ID").unwrap_or("0".to_string()).parse::<u16>().unwrap_or(0);
}

/// Twitter Snowflake-like ID generator
/// first bit 0
/// next 41 bits are milliseconds
/// next 10 bits are generator id
/// last 12 bits are sequence id
fn construct_id(millis: u64, machine_id: u16, sequence_id: u16) -> u64 {
    let b10 = 0b1111111111;
    let b12 = 0b111111111111;

    (millis << 22) | ((machine_id & b10) << 12) as u64 | (sequence_id & b12) as u64
}

fn millis_41bits() -> u64 {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    millis & 0x1FF_FFFF_FFFF
}

pub fn get_next_id() -> u64 {
    let id = GENERATOR_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if id == -1 {
        println!("Generator thread starting...");
        GENERATOR_MILLIS.store(millis_41bits(), std::sync::atomic::Ordering::SeqCst);
        start_reset_thread();
    }

    let millis = GENERATOR_MILLIS.load(std::sync::atomic::Ordering::SeqCst);
    
    construct_id(millis, *MACHINE_ID, id as u16)
}

fn start_reset_thread() {
    std::thread::spawn(|| {
        let spin_sleeper = spin_sleep::SpinSleeper::new(100_000)
            .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);
        let mut inactivity = 0;
        let mut last_millis = millis_41bits();

        loop {
            spin_sleeper.sleep(std::time::Duration::from_micros(1000));

            if GENERATOR_COUNT.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                inactivity = 0;
            } else {
                inactivity += 1;
                if inactivity >= globals::INACTIVITY_SLEEP {
                    let val = GENERATOR_COUNT.compare_exchange(
                        0,
                        -1,
                        std::sync::atomic::Ordering::SeqCst,
                        std::sync::atomic::Ordering::SeqCst,
                    );

                    if val.is_ok() {
                        println!("Generator thread ending...");
                        return;
                    }
                }
            }

            let millis = millis_41bits();
            if millis > last_millis {
                last_millis = millis;
                GENERATOR_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
                GENERATOR_MILLIS.store(millis, std::sync::atomic::Ordering::SeqCst);
            }
        }
    });
}
