use std::env;

#[derive(Debug)]
pub struct ParsedEntry {
    pub subcommand: String,
    pub forwarded: Vec<String>,
}

pub fn parse_entry(args: &[String]) -> Result<ParsedEntry, i32> {
    let mut arg_index = 1usize;
    while arg_index < args.len() {
        let flag = args[arg_index].as_str();
        match flag {
            "--verbose" | "-v" => {
                env::set_var("SPEC_RUNNER_DEBUG", "1");
                env::set_var("SPEC_RUNNER_DEBUG_LEVEL", "1");
                arg_index += 1;
            }
            "-vv" => {
                env::set_var("SPEC_RUNNER_DEBUG", "1");
                env::set_var("SPEC_RUNNER_DEBUG_LEVEL", "2");
                arg_index += 1;
            }
            "-vvv" => {
                env::set_var("SPEC_RUNNER_DEBUG", "1");
                env::set_var("SPEC_RUNNER_DEBUG_LEVEL", "3");
                arg_index += 1;
            }
            "--profile-level" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --profile-level requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_PROFILE_LEVEL", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--profile-out" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --profile-out requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_PROFILE_OUT", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--profile-summary-out" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --profile-summary-out requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_PROFILE_SUMMARY_OUT", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--profile-heartbeat-ms" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --profile-heartbeat-ms requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_PROFILE_HEARTBEAT_MS", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--profile-stall-threshold-ms" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --profile-stall-threshold-ms requires value");
                    return Err(2);
                }
                env::set_var(
                    "SPEC_RUNNER_PROFILE_STALL_THRESHOLD_MS",
                    args[arg_index + 1].clone(),
                );
                arg_index += 2;
            }
            "--liveness-level" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --liveness-level requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_LIVENESS_LEVEL", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--liveness-stall-ms" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --liveness-stall-ms requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_LIVENESS_STALL_MS", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--liveness-min-events" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --liveness-min-events requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_LIVENESS_MIN_EVENTS", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--liveness-hard-cap-ms" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --liveness-hard-cap-ms requires value");
                    return Err(2);
                }
                env::set_var("SPEC_RUNNER_LIVENESS_HARD_CAP_MS", args[arg_index + 1].clone());
                arg_index += 2;
            }
            "--liveness-kill-grace-ms" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("ERROR: --liveness-kill-grace-ms requires value");
                    return Err(2);
                }
                env::set_var(
                    "SPEC_RUNNER_LIVENESS_KILL_GRACE_MS",
                    args[arg_index + 1].clone(),
                );
                arg_index += 2;
            }
            _ => break,
        }
    }

    if args.len() <= arg_index {
        eprintln!("ERROR: missing runner adapter subcommand");
        return Err(2);
    }

    Ok(ParsedEntry {
        subcommand: args[arg_index].clone(),
        forwarded: args[(arg_index + 1)..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parse_entry_reads_subcommand_and_forwarded_args() {
        let args = argv(&["spec_runner_cli", "--verbose", "style-check", "--foo", "bar"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "style-check");
        assert_eq!(parsed.forwarded, vec!["--foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn parse_entry_rejects_missing_flag_value() {
        let args = argv(&["spec_runner_cli", "--profile-level"]);
        let code = parse_entry(&args).expect_err("should fail");
        assert_eq!(code, 2);
    }

    #[test]
    fn parse_entry_rejects_missing_subcommand() {
        let args = argv(&["spec_runner_cli", "--verbose"]);
        let code = parse_entry(&args).expect_err("should fail");
        assert_eq!(code, 2);
    }
}
