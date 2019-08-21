//# rand = "0.5.0"
//# dtoa = { git = "https://github.com/dtolnay/dtoa.git" }

fn main() -> std::io::Result<()> {
    let mut buf = Vec::new();
    dtoa::write(&mut buf, 2.71828f64)?;

    println!("{:?}", buf);

    Ok(())
}
