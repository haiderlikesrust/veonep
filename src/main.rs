fn main() -> Result<(), veonep::error::VeonError> {
    let source = r#"
    "Hey" + "WOW"
    "#;
    let mut tokens = veonep::scanner::Scanner::new(source.to_owned());
    println!("{:?}", tokens.tokenize()?);
    Ok(())
}
