fn main() {
    let env_vars = get_env_vars();

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=api/migrations");

    if let Some(env_vars) = env_vars {
        for env_var in env_vars {
            println!("cargo:rustc-env={}={}", &env_var.0, &env_var.1);
        }
    }
}

fn get_env_vars() -> Option<Vec<(String, String)>> {
    dotenvy::dotenv().ok()?;

    let env_vars: Vec<(String, String)> = dotenvy::vars().collect();
    Some(env_vars)
}
