// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fs::{read_to_string, write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod dump;
mod strip;
mod utils;

const EXIT_CODE_ERROR: i32 = 1;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to operate file: {0}")]
    FileIo(#[from] std::io::Error),
    #[error("{0}")]
    DumpCpuConfig(#[from] dump::Error),
    #[error("Failed to serialize/deserialize JSON file: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Failed to strip CPU configuration: {0}")]
    StripCpuConfig(#[from] strip::Error),
    #[error("{0}")]
    Utils(#[from] utils::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Dump CPU configuration in custom CPU template format.
    Dump {
        /// Path of firecracker config file.
        #[arg(short, long, value_name = "PATH")]
        config: PathBuf,
        /// Path of output file.
        #[arg(short, long, value_name = "PATH", default_value = "cpu_config.json")]
        output: PathBuf,
    },
    /// Strip items shared between multiple CPU configurations.
    Strip {
        /// List of paths of input CPU configuration files.
        #[arg(short, long, num_args = 2..)]
        path: Vec<PathBuf>,
        /// Suffix of output files. To overwrite input files, specify an empty string ''.
        #[arg(short, long, default_value = "_stripped")]
        suffix: String,
    },
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Dump { config, output } => {
            let config = read_to_string(config)?;
            let (vmm, _) = utils::build_microvm_from_config(&config)?;

            let cpu_config = dump::dump(vmm)?;

            let cpu_config_json = serde_json::to_string_pretty(&cpu_config)?;
            write(output, cpu_config_json)?;
        }
        Command::Strip { path, suffix } => {
            let input = path
                .iter()
                .map(read_to_string)
                .collect::<std::io::Result<Vec<_>>>()?;

            let strip_result = strip::strip(input)?;

            let path = path
                .iter()
                .map(|path| utils::add_suffix(path, &suffix))
                .collect::<Vec<_>>();
            for (path, result) in path.into_iter().zip(strip_result.into_iter()) {
                write(path, result)?;
            }
        }
    };

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(EXIT_CODE_ERROR);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use ::utils::tempfile::TempFile;
    use vmm::utilities::mock_resources::kernel_image_path;

    use super::*;

    pub fn generate_config(kernel_image_path: &str, rootfs_path: &str) -> String {
        format!(
            r#"{{
                "boot-source": {{
                    "kernel_image_path": "{}"
                }},
                "drives": [
                    {{
                        "drive_id": "rootfs",
                        "path_on_host": "{}",
                        "is_root_device": true,
                        "is_read_only": false
                    }}
                ]
            }}"#,
            kernel_image_path, rootfs_path,
        )
    }

    fn generate_config_file(kernel_image_path: &str, rootfs_path: &str) -> TempFile {
        let config = generate_config(kernel_image_path, rootfs_path);
        let config_file = TempFile::new().unwrap();
        config_file.as_file().write_all(config.as_bytes()).unwrap();
        config_file
    }

    #[test]
    fn test_dump_command() {
        let kernel_image_path = kernel_image_path(None);
        let rootfs_file = TempFile::new().unwrap();
        let config_file =
            generate_config_file(&kernel_image_path, rootfs_file.as_path().to_str().unwrap());
        let output_file = TempFile::new().unwrap();

        let args = vec![
            "cpu-template-helper",
            "dump",
            "--config",
            config_file.as_path().to_str().unwrap(),
            "--output",
            output_file.as_path().to_str().unwrap(),
        ];
        let cli = Cli::parse_from(args);

        run(cli).unwrap();
    }

    #[cfg(target_arch = "x86_64")]
    fn build_sample_cpu_config_files() -> Vec<TempFile> {
        let files = vec![TempFile::new().unwrap(), TempFile::new().unwrap()];
        files[0]
            .as_file()
            .write_all(
                r#"{
                    "cpuid_modifiers": [
                        {
                            "leaf": "0x0",
                            "subleaf": "0x0",
                            "flags": 0,
                            "modifiers": [
                                {
                                    "register": "eax",
                                    "bitmap": "0b00000000000000000000000000000000"
                                }
                            ]
                        }
                    ],
                    "msr_modifiers": [
                        {
                            "addr": "0x0",
                            "bitmap": "0b0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }"#
                .as_bytes(),
            )
            .unwrap();
        files[1]
            .as_file()
            .write_all(
                r#"{
                    "cpuid_modifiers": [
                        {
                            "leaf": "0x0",
                            "subleaf": "0x0",
                            "flags": 0,
                            "modifiers": [
                                {
                                    "register": "eax",
                                    "bitmap": "0b00000000000000000000000000000000"
                                }
                            ]
                        }
                    ],
                    "msr_modifiers": [
                        {
                            "addr": "0x0",
                            "bitmap": "0b0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }"#
                .as_bytes(),
            )
            .unwrap();
        files
    }

    #[cfg(target_arch = "aarch64")]
    fn build_sample_cpu_config_files() -> Vec<TempFile> {
        let files = vec![TempFile::new().unwrap(), TempFile::new().unwrap()];
        files[0]
            .as_file()
            .write_all(
                r#"{
                    "reg_modifiers": [
                        {
                            "addr": "0x0",
                            "bitmap": "0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }"#
                .as_bytes(),
            )
            .unwrap();
        files[1]
            .as_file()
            .write_all(
                r#"{
                    "reg_modifiers": [
                        {
                            "addr": "0x0",
                            "bitmap": "0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }"#
                .as_bytes(),
            )
            .unwrap();
        files
    }

    #[test]
    fn test_strip_command() {
        let files = build_sample_cpu_config_files();

        let mut args = vec!["cpu-template-helper", "strip", "-p"];
        let paths = files
            .iter()
            .map(|file| file.as_path().to_str().unwrap())
            .collect::<Vec<_>>();
        args.extend(paths);
        let cli = Cli::parse_from(args);

        run(cli).unwrap();
    }
}
