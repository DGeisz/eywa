use eywa::eywa::get_uuid;

fn main() {
    let my_uuid = get_uuid();
    let second = my_uuid.clone();
    println!("This is uuid: {}", second);
}