fn main() {
    std::process::exit(risk_analyzer::cli::run(std::env::args_os().skip(1)));
}
