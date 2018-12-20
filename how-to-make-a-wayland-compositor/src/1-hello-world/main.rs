extern crate wlroots;

use wlroots::{compositor, utils::log::{WLR_DEBUG, init_logging}};

fn main() {
    init_logging(WLR_DEBUG, None);
    compositor::Builder::new().build_auto(()).run()
}
