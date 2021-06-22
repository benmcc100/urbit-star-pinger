use std::process::Command;
use std::{thread, time};
use rand::Rng;
use mysql::*;
use mysql::prelude::*;
use std::convert::TryInto;

fn main() {
    let mut stars_to_ping: Vec<String> = Vec::new();
    let mut pinged_stars: Vec<String> = Vec::new();

    // mysql connection
    let url = "DB URL HERE";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();

    conn.query_map(
        "SELECT point_name FROM stars WHERE tracked = true",
        |star| {
            stars_to_ping.push(star);
        },
    ).unwrap();

    println!("will ping stars: {:#?}", stars_to_ping);
    let star_count = stars_to_ping.len();
    
    // main program loop
    // sleep time is calculated by amount of stars in pinging pool
    // designed to go through whole pool each hour
    let sleep_time = time::Duration::from_secs((3600 / star_count).try_into().unwrap());
    loop {
        // choose a star at random from the list and ping it
        // store the results of the ping in a file named for the star
        let mut rng = rand::thread_rng();
        // if no stars left put the pinged stars back in rotation
        if stars_to_ping.is_empty() {
            stars_to_ping = pinged_stars.clone();
            pinged_stars = Vec::new();
        }
        let star_num = rng.gen_range(0..stars_to_ping.len());
        let star = stars_to_ping.remove(star_num);
        pinged_stars.push(star.to_string());
        let _handle = thread::spawn({
            let pool = pool.clone();
            move || {
            let conn = pool.get_conn().unwrap();
            ping_star(&star[..], conn);
            }
        });
        // wait
        thread::sleep(sleep_time);
    }
}

fn update_db(star: &str, online: bool, mut conn: PooledConn) {
    if online {
        let _res = conn.exec_drop(
            "UPDATE stars SET status = True WHERE point_name = :star_name",
            params! {
                "star_name" => &star[..],
            },
        ).unwrap();
        println!("pinged {} successfully", star);
    } else {
        let _res = conn.exec_drop(
            "UPDATE stars SET status = False WHERE point_name = :star_name",
            params! {
                "star_name" => &star[..],
            },
        ).unwrap();
        println!("pinged {} UNsuccessfully", star);
    }
}

fn ping_star(star: &str, conn: PooledConn) {
    let _hi = Command::new("tmux")
                     .arg("send")
                     .arg("-t")
                     .args(&["bacrys", &format!("|hi ~{}", star), "ENTER"])
                     .output()
                     .expect("bad command");

    thread::sleep(time::Duration::from_secs(1));
    let mut tries = 0;
    while tries < 10 {
        let res = Command::new("tmux")
                      .arg("capture-pane")
                      .arg("-t")
                      .args(&["bacrys", "-p", "-S", "25"])
                      .output()
                      .expect("bad command");

        for line in String::from_utf8(res.stdout).unwrap().lines() {
            if line == format!("hi ~{} successful", star) {
                update_db(star, true, conn);
                return;
            }
        }
        println!("waiting on star {}", star);
        tries += 1;
        thread::sleep(time::Duration::from_secs(60));
    }
    update_db(star, false, conn);
}