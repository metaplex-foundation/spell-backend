const SKIP_SOLANA_TESTS: &str = "SKIP_SOLANA_TESTS";
const SOLANA_HOME: &str = "SOLANA_HOME";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed={}", SKIP_SOLANA_TESTS);

    // We may want to explicitly skip solana tests (e.g. for CI)
    // Or if solana-test-validator is unavailable, tests won;t run
    if get_bool_env(SKIP_SOLANA_TESTS)
        || std::env::var(SOLANA_HOME).is_err() && which::which("solava-test-validator").is_err()
    {
        println!("cargo:rustc-cfg=skip_solana_tests");
    }

    Ok(())
}

fn get_bool_env(var_name: &str) -> bool {
    let Ok(var_value) = std::env::var(var_name) else {
        return false;
    };
    var_value.parse::<bool>().unwrap_or(false)
}
