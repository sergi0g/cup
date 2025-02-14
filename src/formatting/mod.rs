pub mod spinner;

use rustc_hash::FxHashMap;

use crate::{
    structs::{
        status::Status,
        update::{Update, UpdateInfo},
    },
    utils::{json::to_simple_json, sort_update_vec::sort_update_vec},
};

pub fn print_updates(updates: &[Update], icons: &bool) {
    let sorted_updates = sort_update_vec(updates);
    let updates_by_server = {
        let mut servers: FxHashMap<&str, Vec<&Update>> = FxHashMap::default();
        sorted_updates.iter().for_each(|update| {
            let key = update.server.as_deref().unwrap_or("");
            match servers.get_mut(&key) {
                Some(server) => server.push(update),
                None => {
                    let _ = servers.insert(key, vec![update]);
                }
            }
        });
        servers
    };
    for (server, updates) in updates_by_server {
        if server.is_empty() {
            println!("\x1b[90;1m~ Local images\x1b[0m")
        } else {
            println!("\x1b[90;1m~ {}\x1b[0m", server)
        }
        let (reference_width, status_width, time_width) =
            updates.iter().fold((9, 6, 9), |acc, update| {
                let reference_length = update.reference.len();
                let status_length = update.get_status().to_string().len()
                    + match &update.result.info {
                        UpdateInfo::Version(info) => {
                            info.current_version.len() + info.new_version.len() + 6
                        }
                        _ => 0,
                    };
                let time_length = update.time.to_string().len();
                return (
                    if reference_length > acc.0 {
                        reference_length
                    } else {
                        acc.0
                    },
                    if status_length > acc.1 {
                        status_length
                    } else {
                        acc.1
                    },
                    if time_length > acc.2 {
                        time_length
                    } else {
                        acc.2
                    },
                );
            });
        println!(
            " \x1b[90;1m╭{:─<rw$}┬{:─<sw$}┬{:─<tw$}╮\x1b[0m",
            "",
            "",
            "",
            rw = reference_width,
            sw = status_width + {
                if *icons {
                    2
                } else {
                    0
                }
            },
            tw = time_width
        );
        println!(
            " \x1b[90;1m│\x1b[36;1m{:<rw$}\x1b[90;1m│\x1b[36;1m{:<sw$}\x1b[90;1m│\x1b[36;1m{:<tw$}\x1b[90;1m│\x1b[0m",
            "Reference",
            "Status",
            "Time (ms)",
            rw = reference_width,
            sw = status_width + {
                if *icons {
                    2
                } else {
                    0
                }
            },
            tw = time_width
        );
        println!(
            " \x1b[90;1m├{:─<rw$}┼{:─<sw$}┼{:─<tw$}┤\x1b[0m",
            "",
            "",
            "",
            rw = reference_width,
            sw = status_width + {
                if *icons {
                    2
                } else {
                    0
                }
            },
            tw = time_width
        );
        for update in updates {
            let status = update.get_status();
            let icon = if *icons {
                match status {
                    Status::UpToDate => "\u{f058} ",
                    Status::Unknown(_) => "\u{f059} ",
                    _ => "\u{f0aa} ",
                }
            } else {
                ""
            };
            let color = match status {
                Status::UpdateAvailable | Status::UpdatePatch => "\x1b[34m",
                Status::UpdateMinor => "\x1b[33m",
                Status::UpdateMajor => "\x1b[31m",
                Status::UpToDate => "\x1b[32m",
                Status::Unknown(_) => "\x1b[90m",
            };
            let description = format!(
                "{} {}",
                status.to_string(),
                match &update.result.info {
                    UpdateInfo::Version(info) => {
                        format!("({} → {})", info.current_version, info.new_version)
                    }
                    _ => String::new(),
                }
            );
            println!(
                " \x1b[90;1m│\x1b[0m{:<rw$}\x1b[90;1m│\x1b[0m{}{}{:<sw$}\x1b[0m\x1b[90;1m│\x1b[0m{:<tw$}\x1b[90;1m│\x1b[0m",
                update.reference,
                color,
                icon,
                description,
                update.time,
                rw = reference_width,
                sw = status_width,
                tw = time_width
            );
        }
        println!(
            " \x1b[90;1m╰{:─<rw$}┴{:─<sw$}┴{:─<tw$}╯\x1b[0m",
            "",
            "",
            "",
            rw = reference_width,
            sw = status_width + {
                if *icons {
                    2
                } else {
                    0
                }
            },
            tw = time_width
        );
    }
}

pub fn print_raw_updates(updates: &[Update]) {
    println!("{}", to_simple_json(updates));
}
