use super::Guard;

fn unique_name(tag: &str) -> String {
    format!("Local\\CuaKeyTest-{tag}-{}", std::process::id())
}

#[test]
fn single_instance_second_acquire_fails() {
    let name = unique_name("second");

    let first = Guard::acquire(&name).unwrap();
    let second = Guard::acquire(&name).unwrap();

    assert!(first.is_some(), "the first instance gets the guard");
    assert!(second.is_none(), "a second one finds it taken");
}

#[test]
fn guard_is_released_on_drop() {
    let name = unique_name("drop");

    drop(Guard::acquire(&name).unwrap());
    let again = Guard::acquire(&name).unwrap();

    assert!(again.is_some(), "the name is free once the guard is gone");
}
