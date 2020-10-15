use eywa::eywa::get_uuid;
use std::collections::HashMap;

fn main() {
    let my_uuid = get_uuid();
    let second = my_uuid.clone();

    let mut map = HashMap::new();

    let vector = vec![1.1, 2.3, 3.5];

    map.insert(format!("{:?}", vector), "help");

    let vector2 = vec![1.1, 2.3, 3.5];

    let a = 1.clone();
    match map.get(&format!("{:?}", vector2)) {
        Some(strang) => println!("Got this string {}", strang),
        None => println!("Didn't get anything")
    }
    println!("This is vec: {:?}", vector);
    println!("This is uuid: {}", second);
}