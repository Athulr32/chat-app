use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_time_in_seconds() ->u64{
    let time = SystemTime::now();
    let since_the_epoch = time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let current_time =
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

        current_time
}
