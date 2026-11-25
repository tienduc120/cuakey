use std::time::Duration;

use super::double_click_time;

#[test]
fn the_double_click_time_is_a_positive_span_the_system_allows() {
    let time = double_click_time();

    assert!(
        time > Duration::ZERO,
        "a double click time of zero would make every click single"
    );
    assert!(
        time <= Duration::from_secs(5),
        "the system holds the double click time to 5 seconds at most, got {time:?}"
    );
}
