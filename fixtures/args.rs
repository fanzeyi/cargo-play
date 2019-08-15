fn main() {
    println!("{}", std::env::args().skip(1).next().unwrap());
}
