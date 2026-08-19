use chessr::uci::uci_loop;

fn main() {
    std::panic::set_hook(Box::new(|info| {
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/chessr-panic.log") {
        use std::io::Write;
        let _ = writeln!(f, "{info}");
    }
}));
    uci_loop();
}
