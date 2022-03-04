//# rand = "0.5.0"
//# dtoa = { git = "https://github.com/dtolnay/dtoa.git" }

fn main() -> std::io::Result<()> {
    let mut buffer = dtoa::Buffer::new();
    let printed = buffer.format(2.71828f64);
    assert_eq!(printed, "2.71828");

    Ok(())
}
