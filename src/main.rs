use std::fs::File;
use std::io::prelude::*;
use std::process::{Command, Stdio};

use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder,
};

use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // Read config file
    let mut cfg_file = File::open("config.json").expect("Failed to open config.json");
    let mut config = String::new();
    cfg_file.read_to_string(&mut config).expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config).expect("Failed to parse config.json");

    // Initialize client
    let mut client = Client::builder(&config.token)
        .event_handler(Handler)
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client");

    // Add the command that we use to check the code to the client's data
    client.data.write().await.insert::<CmdKey>(config.cmd);

    client.start().await.expect("Error starting client");
}

/// Helper macro that handles a Result by printing an error message and returning
macro_rules! err_ret {
    ($msg:expr, $expression:expr) => {
        match $expression {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error {}: {:?}", $msg, e);
                return;
            },
        }
    }
}

/// Helper macro that handles an Option by printing an error message and returning
macro_rules! opt_ret {
    ($msg:expr, $expression:expr) => {
        match $expression {
            Some(r) => r,
            _ => {
                eprintln!("Error {}", $msg);
                return;
            },
        }
    }
}

/// Event handler
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        // If the message was sent by this bot, ignore it
        if context.data.read().await.get::<UserIdKey>() == Some(&msg.author.id.0) {
            return;
        }

        // List of all Python code blocks in this message
        let mut blocks: Vec<String> = Vec::new();
        // The current Python code block being read
        let mut cur_block = String::new();
        // Whether or not we are currently in *any* code block
        let mut in_block = false;
        // Whether or not we are currently in a Python code block
        let mut in_py_block = false;

        for line in msg.content.lines() {
            if in_block {
                if line.starts_with("```") {
                    // End of code block
                    in_block = false;
                    if in_py_block {
                        in_py_block = false;
                        // Add the current block to the list and reset it
                        blocks.push(cur_block);
                        cur_block = String::new();
                    }
                } else {
                    if in_py_block {
                        // Add the current line to the current block
                        cur_block.push_str(line);
                        cur_block.push_str("\n");
                    }
                }
            } else {
                if line.starts_with("```") {
                    // Beginning of a code block
                    in_block = true;
                    // Check the language of the code block to see if it's Python
                    let lang = &line[3..].to_lowercase();
                    if lang == "py" || lang == "python" {
                        in_py_block = true;
                    }
                }
            }
        }

        if !blocks.is_empty() {
            // Get the command used to check the code from the context data
            let data = context.data.read().await;
            let cmd = opt_ret!("getting command", data.get::<CmdKey>());

            // Get the author's nickname if possible, otherwise default to their username
            let name = msg.author_nick(&context).await;
            let name = name.as_ref().unwrap_or(&msg.author.name);

            // Generate response, starting with overly long and annoying header
            let mut response = String::new();
            let fstr = format!("! You posted a message with {} Python code blocks.", blocks.len());
            response.push_str(&MessageBuilder::new()
                .push_safe("Hi, ")
                .push_bold_safe(name)
                .push_safe(
                    if blocks.len() == 1 {
                        "! You posted a message with 1 Python code block."
                    } else {
                        &fstr
                    }
                )
                .push_line_safe(" Let's check your code for PEP8 style issues!")
                .build());

            for (i, block) in (0usize..).zip(blocks) {
                // Create child process to run command
                let mut child = err_ret!("running command", Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                );

                // Get stdin from the child process and send it the code
                let mut stdin = opt_ret!("opening stdin of command", child.stdin.take());
                err_ret!("writing to stdin of command", stdin.write_all(block.as_bytes()));

                // Dropping the handle closes it
                drop(stdin);

                // Read output from the command
                let output = err_ret!("reading output of command", child.wait_with_output());
                let output = String::from_utf8_lossy(&output.stdout);

                if output == "" {
                    // If the output is empty, we have no style issues
                    response.push_str(&MessageBuilder::new()
                        .push_bold_line_safe(format!("Codeblock {}: No style issues here!", i+1))
                        .build());
                } else {
                    // Otherwise, add a codeblock with the output
                    response.push_str(&MessageBuilder::new()
                        .push_bold_line_safe(format!("Codeblock {}:", i+1))
                        .push_codeblock_safe(output, None)
                        .build());
                }
            }

            // Send the message
            err_ret!("sending message", msg.reply_ping(&context.http, &response).await);
        }
    }

    async fn ready(&self, context: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Add the bot's user ID to the client data;
        // this way we can use it to check if a message was sent by this bot
        context.data.write().await.insert::<UserIdKey>(ready.user.id.0);
    }
}

/// Configuration data for the bot, loaded from config.json
#[derive(Serialize, Deserialize)]
struct Config {
    /// Discord bot token
    token: String,
    /// The command to run in order to check the code;
    /// should read code as input from stdin
    /// and write output to stdout
    #[serde(default = "default_cmd")]
    cmd: Vec<String>,
}

/// Provides a default command, if none is given explicitly in the config
fn default_cmd() -> Vec<String> {
    vec!["flake8".to_string(), "--stdin-display-name".to_string(), "block".to_string(), "-".to_string()]
}

/// TypeMap key for the user ID of the bot account
struct UserIdKey;

impl TypeMapKey for UserIdKey {
    type Value = u64;
}

/// TypeMap key for the command used to check the code
struct CmdKey;

impl TypeMapKey for CmdKey {
    type Value = Vec<String>;
}
