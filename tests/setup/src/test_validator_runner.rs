use base64::Engine;
use solana_sdk::pubkey::Pubkey;
use std::{
    env,
    fs::File,
    io::Write,
    ops::Deref,
    path::Path,
    process::{Child, Command},
};

const PROGRAM_NAME: &'static str = "solana-test-validator";
const ENV_SOLANA_HOME: &'static str = "SOLANA_HOME";

/// This wrapper allow to launch `solana-test-validator` as separate process,
/// plus specify list of contracts and account to deploy on startup.
pub struct TestValidatorRunner {
    port: u32,
    contracts: Vec<ContractToDeploy>,
    clone_contracts: Vec<String>,
    accounts: Vec<AccountInit>,
    search_paths: Vec<String>,
}

impl TestValidatorRunner {
    // The thing is that after the server shuts down and releases the port,
    // this port doesn't become available immediately.
    // That's why for each test we need to either specify different port
    // for solana-test-validator, or wait fo the port to become available.
    // Here, we went with the first option.
    pub fn new(port: u32) -> TestValidatorRunner {
        TestValidatorRunner {
            port: port,
            contracts: Vec::new(),
            clone_contracts: Vec::new(),
            accounts: Vec::new(),
            search_paths: Vec::new(),
        }
    }

    pub fn add_account(&mut self, account: &AccountInit) {
        self.accounts.push(account.clone());
    }

    pub fn add_program(&mut self, program: &ContractToDeploy) {
        self.contracts.push(program.clone());
    }

    pub fn clone_program(&mut self, program_id: &str) {
        self.clone_contracts.push(program_id.to_string());
    }

    #[allow(clippy::expect_fun_call)]
    pub fn run(&self) -> std::io::Result<SolanaProcess> {
        // If program is not an absolute path, the PATH will be searched in an OS-defined way.
        let cmd_name = if std::env::var(ENV_SOLANA_HOME).is_ok() {
            Path::new(&std::env::var(ENV_SOLANA_HOME).unwrap())
                .join(PROGRAM_NAME)
                .to_str()
                .unwrap()
                .to_string()
        } else {
            PROGRAM_NAME.to_string()
        };
        let mut cmd = Command::new(cmd_name);
        cmd.arg("--reset");
        cmd.args(["--url", "devnet"]);

        let port_string = self.port.to_string();
        cmd.arg("--rpc-port").arg(&port_string);
        cmd.arg("--faucet-port").arg((&self.port + 1).to_string());

        for contract in &self.contracts {
            let path_to_so = self
                .find_in_paths(&contract.path)
                .expect(&format!("Cannot find: {}", &contract.path));
            cmd.args(["--bpf-program", &contract.addr.to_string(), &path_to_so]);
        }

        for contract in &self.clone_contracts {
            cmd.args(["--clone-upgradeable-program", contract]);
        }

        for account in &self.accounts {
            let file_path = write_to_temp_file(&port_string, &account.name, account.to_json().as_bytes());
            cmd.args(["--account", &account.pubkey.to_string(), &file_path]);
        }

        cmd.arg("--quiet");
        if cfg!(target_os = "macos") {
            eprintln!("As you running macOS test validator could fail, so we adding path to 'gnu-tar' to env.");
            eprintln!("Also please check that you have 'gnu-tar' installed!");
            let current_path = env::var("PATH").expect("Cannot extract 'PATH'!");
            let path_with_gnu_tar = format!("/opt/homebrew/opt/gnu-tar/libexec/gnubin:{current_path}");
            cmd.env("PATH", path_with_gnu_tar);
        }

        Ok(SolanaProcess { solana_url: format!("http://127.0.0.1:{}", self.port), process: cmd.spawn()? })
    }

    fn find_in_paths(&self, file: &str) -> Option<String> {
        // if the given path to the contract is absolute,
        // or relative to the current dir, i.e. already accessible
        if Path::new(file).exists() {
            return Some(file.to_string());
        }

        for search_path in &self.search_paths {
            let try_path = Path::new(search_path).join(file);
            if try_path.exists() {
                return try_path.to_str().map(|s| s.to_owned());
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct ContractToDeploy {
    pub addr: Pubkey,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct AccountInit {
    pub name: String,
    pub pubkey: Pubkey,
    pub data: Vec<u8>,
    pub owner: Pubkey,
}

impl AccountInit {
    pub fn to_json(&self) -> String {
        let pubkey = self.pubkey;
        let data = base64::prelude::BASE64_STANDARD.encode(&self.data);
        let owner = self.owner;
        let space = self.data.len();
        format!(
            r#"
        {{
            "pubkey": "{pubkey}",
            "account": {{
              "lamports": 10000000000000000,
              "data": [
                "{data}",
                "base64"
              ],
              "owner": "{owner}",
              "executable": false,
              "rentEpoch": 18446744073709551615,
              "space": {space}
            }}
        }}
        "#
        )
    }
}

fn write_to_temp_file(temp_prefix: &str, name: &str, payload: &[u8]) -> String {
    let dir = std::env::temp_dir();
    let accounts_temp_dir = dir.join("test_sol_programs");
    if !accounts_temp_dir.exists() {
        std::fs::create_dir(&accounts_temp_dir).unwrap();
    }
    let the_test_accounts_dir = accounts_temp_dir.join(temp_prefix);
    if !the_test_accounts_dir.exists() {
        std::fs::create_dir(&the_test_accounts_dir).unwrap();
    }
    let file_path = the_test_accounts_dir.join(name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(payload).unwrap();
    file_path.to_str().unwrap().to_string()
}

pub struct SolanaProcess {
    pub solana_url: String,
    pub process: Child,
}

impl Drop for SolanaProcess {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}

impl Deref for SolanaProcess {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.process
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_to_json() {
        let acc = AccountInit {
            name: "registrar.json".to_string(),
            pubkey: Pubkey::from_str("7KXf5wqxoDE9QTDdVysHULruroRCemWU9WQEyDcRkUFC").unwrap(),
            data: vec![1, 2, 3],
            owner: Pubkey::from_str("3GepGwMp6WgPqgNa5NuSpnw3rQjYnqHCcVWhVmpGnw6s").unwrap(),
        };
        println!("{}", acc.to_json());
    }
}
