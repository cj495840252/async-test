use std::sync::Mutex;

fn main() {
    let mut a = Mutex::new(Some(0));
    let mut b = a.lock().unwrap();

    while true {
        if let Some(i) = b.take(){
            println!("s: {i}");
            // *b = Some(i+1);
        }
    }
}