extern crate env_logger;
extern crate shine_store;
extern crate shine_testutils;

use std::env;
use std::thread;

use shine_store::spscstate::*;
use shine_testutils::*;

const ITER_COUNT: i32 = 0x2ffff;

#[test]
fn single_threaded_logic() {
    init_test_logger(module_path!());

    let (p, c) = state_channel();

    assert!(c.receive().is_err());

    p.send(1).unwrap();
    assert_eq!(c.receive().unwrap(), 1);
    assert!(c.receive().is_err());
    assert!(c.receive().is_err());

    p.send(2).unwrap();
    assert_eq!(c.receive().unwrap(), 2);
    assert!(c.receive().is_err());
    assert!(c.receive().is_err());

    p.send(3).unwrap();
    assert_eq!(c.receive().unwrap(), 3);
    assert!(c.receive().is_err());
    assert!(c.receive().is_err());

    p.send(4).unwrap();
    assert_eq!(c.receive().unwrap(), 4);
    assert!(c.receive().is_err());
    assert!(c.receive().is_err());
}

#[test]
fn single_threaded_stress_small_buffer() {
    init_test_logger(module_path!());

    let (p, c) = state_channel();

    for x in 0..ITER_COUNT {
        p.send(x).unwrap();
        assert_eq!(c.receive().unwrap(), x);
    }
}

#[test]
fn multi_threaded_stress_small_buffer() {
    init_test_logger(module_path!());

    assert!(
        env::var("RUST_TEST_THREADS").unwrap_or("0".to_string()) == "1",
        "This test shall run in single threaded test environment: RUST_TEST_THREADS=1"
    );

    let (p, c) = state_channel();

    let tp = thread::spawn(move || {
        for x in 0..ITER_COUNT {
            p.send(x).unwrap();
        }
        //println!("produced: {}", ITER_COUNT);
    });
    let tc = thread::spawn(move || {
        let mut prev = -1;
        let mut _cnt = 0;
        loop {
            match c.receive() {
                Ok(x) => {
                    _cnt += 1;
                    assert!(prev < x);
                    prev = x;
                    if prev == ITER_COUNT - 1 {
                        break;
                    }
                }
                Err(_) => {}
            }
        }
        //println!("consumed: {}", _cnt);
    });

    tp.join().unwrap();
    tc.join().unwrap();
}

fn is_prime(n: i32) -> bool {
    if n == 2 || n == 3 {
        return true;
    } else if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let mut i = 5;
    let mut res = true;
    while i * i <= n {
        if n % i == 0 {
            res = false;
        }
        i = i + 1;
    }
    res
}

fn long_calc(x: i32) -> i32 {
    if is_prime(x) {
        11
    } else {
        87
    }
}

struct BigData {
    pre: i32,
    x: i32,
    data: [i32; 64],
    post: i32,
}

impl Default for BigData {
    fn default() -> BigData {
        BigData {
            pre: 2,
            x: 0,
            data: [0; 64],
            post: 2,
        }
    }
}

#[test]
fn single_threaded_stress_big_buffer() {
    init_test_logger(module_path!());

    let (p, c) = state_channel::<BigData>();
    for x in 0..ITER_COUNT {
        {
            let mut d = p.send_buffer().unwrap();
            assert_eq!(d.pre, 2);
            d.pre = 1;
            for i in 0..d.data.len() {
                d.data[i] = x;
            }
            assert_eq!(d.post, 2);
            d.post = 1;
        }

        {
            let mut d = c.receive_buffer().unwrap();
            assert_eq!(d.pre, 1);
            assert_eq!(d.post, 1);
            d.pre = 2;
            for i in 0..d.data.len() {
                assert_eq!(d.data[i], x);
            }
            d.post = 2;
        }
    }
}

#[test]
fn multi_threaded_stress_big_buffer() {
    init_test_logger(module_path!());

    assert!(
        env::var("RUST_TEST_THREADS").unwrap_or("0".to_string()) == "1",
        "This test shall run in single threaded test environment: RUST_TEST_THREADS=1"
    );

    let (p, c) = state_channel::<BigData>();

    let tp = thread::spawn(move || {
        for x in 0..ITER_COUNT {
            let mut d = p.send_buffer().unwrap();
            d.pre = 1;
            d.x = x;
            for i in 0..d.data.len() {
                d.data[i] = long_calc(x);
            }
            d.post = 1;
            assert_eq!(d.pre, 1);
            assert_eq!(d.post, 1);
        }
        //println!("produced: {}", ITER_COUNT);
    });
    let tc = thread::spawn(move || {
        let mut prev = -1;
        let mut _cnt = 0;
        loop {
            match c.receive_buffer() {
                Ok(mut d) => {
                    _cnt += 1;
                    assert_eq!(d.pre, 1);
                    assert_eq!(d.post, 1);
                    d.pre = 2;
                    for i in 0..d.data.len() {
                        assert_eq!(d.data[i], d.data[0]);
                    }
                    d.post = 2;
                    assert_eq!(d.pre, 2);
                    assert_eq!(d.post, 2);
                    assert!(prev < d.x);
                    prev = d.x;
                    if prev == ITER_COUNT - 1 {
                        break;
                    }
                }
                Err(_) => {}
            }
        }
        //println!("consumed: {}", _cnt);
    });

    tp.join().unwrap();
    tc.join().unwrap();
}
