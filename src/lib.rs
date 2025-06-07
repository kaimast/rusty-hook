use std::collections::HashMap;

mod config;
pub use config::NO_CONFIG_FILE_FOUND;

mod git;
pub use git::NO_CONFIG_FILE_FOUND_ERROR_CODE;

pub mod hooks;

pub fn init_directory<F, G, H>(
    run_command: F,
    write_file: G,
    file_exists: H,
    target_directory: Option<&str>,
    hook_file_skip_list: Vec<&str>,
) -> Result<(), String>
where
    F: Fn(
        &str,
        Option<&str>,
        bool,
        Option<&HashMap<String, String>>,
    ) -> Result<Option<String>, Option<String>>,
    G: Fn(&str, &str, bool) -> Result<(), String>,
    H: Fn(&str) -> Result<bool, ()>,
{
    let root_directory_path = match git::get_root_directory_path(&run_command, target_directory) {
        Ok(Some(path)) => path,
        _ => return Err(String::from("Failure determining git repo root directory")),
    };
    if git::setup_hooks(
        &run_command,
        &write_file,
        &root_directory_path,
        &hook_file_skip_list,
    )
    .is_err()
    {
        return Err(String::from("Unable to create git hooks"));
    };

    if config::create_default_config_file(&write_file, &file_exists, &root_directory_path).is_err()
    {
        return Err(String::from("Unable to create config file"));
    }

    Ok(())
}

pub fn init<F, G, H>(
    run_command: F,
    write_file: G,
    file_exists: H,
    hook_file_skip_list: Vec<&str>,
) -> Result<(), String>
where
    F: Fn(
        &str,
        Option<&str>,
        bool,
        Option<&HashMap<String, String>>,
    ) -> Result<Option<String>, Option<String>>,
    G: Fn(&str, &str, bool) -> Result<(), String>,
    H: Fn(&str) -> Result<bool, ()>,
{
    init_directory(
        &run_command,
        &write_file,
        &file_exists,
        None,
        hook_file_skip_list,
    )
}

pub fn run<F, G, H, I>(
    run_command: F,
    file_exists: G,
    read_file: H,
    log: I,
    hook_name: &str,
    args: Option<String>,
) -> Result<(), Option<String>>
where
    F: Fn(
        &str,
        Option<&str>,
        bool,
        Option<&HashMap<String, String>>,
    ) -> Result<Option<String>, Option<String>>,
    G: Fn(&str) -> Result<bool, ()>,
    H: Fn(&str) -> Result<String, ()>,
    I: Fn(&str, bool),
{
    let root_directory_path = match git::get_root_directory_path(&run_command, None) {
        Ok(Some(path)) => path,
        _ => {
            return Err(Some(String::from(
                "Failure determining git repo root directory",
            )));
        }
    };

    let config_file_contents =
        config::get_config_file_contents(read_file, file_exists, &root_directory_path).map_err(
            |e| {
                if e == config::NO_CONFIG_FILE_FOUND {
                    Some(e)
                } else {
                    Some(String::from("Failed to parse config file"))
                }
            },
        )?;

    let log_details = config::get_log_setting(&config_file_contents);
    let (script, env_vars) = match (
        config::get_hook_script(&config_file_contents, hook_name),
        args,
    ) {
        (Ok(script), None) => (script, None),
        (Ok(script), Some(a)) => (
            script.replace("%rh!", &a),
            Some(
                vec![("RUSTY_HOOKS_GIT_PARAMS".to_owned(), a)]
                    .into_iter()
                    .collect::<HashMap<String, String>>(),
            ),
        ),
        (Err(err), _) => {
            if err == config::MISSING_CONFIG_KEY {
                return Ok(());
            }
            return Err(Some(String::from("Invalid rusty-hooks config file")));
        }
    };

    let message = format!(
        "[rusty-hooks] Found configured hook: {hook_name}\n[rusty-hooks] Running command: {script}\n"
    );
    log(&message, log_details);

    run_command(
        &script,
        Some(&root_directory_path),
        log_details,
        env_vars.as_ref(),
    )
    .map(|_| ())
}

#[cfg(test)]
mod tests;
