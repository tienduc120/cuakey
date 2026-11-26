use super::trim_working_set;

#[test]
fn trimming_the_working_set_succeeds() {
    assert!(
        trim_working_set().is_ok(),
        "Windows accepts a trim of this process's own working set"
    );
}
