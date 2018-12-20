extern crate mdbook;

use mdbook::MDBook;

fn main() {
    MDBook::load("./")
        .expect("Could not load book source")
        .build()
        .expect("Could not build book");
}
