mod brres;

fn main() {
    match brres::parse_file("test.brres") {
        Ok(_) => {}
        Err(e) => {
            println!("Program unexpectedly failed while parsing brres!\n`{e}`");
        }
    }
}
