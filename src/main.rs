mod brres;

fn main() {
    match brres::parse_file("test.brres") {
        Ok(v) => {}
        Err(e) => {
            println!("Program unexpectedly failed while parsing brres!\n`{e}`");
        }
    }
}
