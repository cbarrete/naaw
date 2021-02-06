use std::collections::HashSet;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Sender};
use std::thread;

const SOCKET_PATH: &str = "/tmp/naaw-socket";

#[derive(Debug)]
enum BspcSubCommand {
    NodeAdd,
    NodeRemove,
}

impl BspcSubCommand {
    fn name(&self) -> &str {
        match self {
            BspcSubCommand::NodeAdd => "node_add",
            BspcSubCommand::NodeRemove => "node_remove",
        }
    }

    fn node_position(&self) -> usize {
        match self {
            BspcSubCommand::NodeAdd => 4,
            BspcSubCommand::NodeRemove => 3,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Node(String);

#[derive(Debug)]
struct State {
    tagged_nodes: HashSet<Node>,
    untagged_nodes: HashSet<Node>,
    tag_shown: bool,
}

impl State {
    fn new() -> Self {
        Self {
            tagged_nodes: HashSet::new(),
            untagged_nodes: HashSet::new(),
            tag_shown: true,
        }
    }
}

#[derive(Debug)]
enum Event {
    AddNode(Node),
    RemoveNode(Node),
    TagNode(Node),
    ShowTag,
}

impl Event {
    fn from_bspc(sub_command: &BspcSubCommand, node_id: &str) -> Self {
        match sub_command {
            BspcSubCommand::NodeAdd => Self::AddNode(Node(String::from(node_id))),
            BspcSubCommand::NodeRemove => Self::RemoveNode(Node(String::from(node_id))),
        }
    }
}

fn subscribe_bspc(sub_command: BspcSubCommand, tx: Sender<Event>) {
    thread::spawn(move || {
        let output = Command::new("bspc")
            .arg("subscribe")
            .arg(sub_command.name())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .stdout
            .unwrap();
        for line in BufReader::new(output).lines() {
            let line = match line {
                Err(err) => {
                    eprintln!("{}", err.to_string());
                    continue;
                }
                Ok(l) => l,
            };
            let node_id = match line.split(' ').nth(sub_command.node_position()) {
                None => {
                    eprintln!("Couldn't parse bspc output");
                    continue;
                }
                Some(node) => node,
            };
            if let Err(err) = tx.send(Event::from_bspc(&sub_command, node_id)) {
                eprintln!("{}", err.to_string());
                continue;
            }
        }
    });
}

fn handle_client_stream(mut stream: UnixStream, tx: Sender<Event>) {
    let mut message = String::new();
    stream.read_to_string(&mut message).unwrap();
    if &message == "show" {
        tx.send(Event::ShowTag).unwrap();
        return;
    }
    if let Some(node) = message.strip_prefix("tag ") {
        tx.send(Event::TagNode(Node(String::from(node)))).unwrap();
        return;
    }
    eprintln!("Unsupported message {}", message);
}

fn subscribe_client(tx: Sender<Event>) {
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_client_stream(stream, tx.clone()),
                Err(err) => {
                    eprintln!("{}", err.to_string());
                    continue;
                }
            }
        }
    });
}

fn bspc_toggle_visibility(node: &Node) {
    Command::new("bspc")
        .arg("node")
        .arg(node.0.as_str())
        .arg("-g")
        .arg("hidden")
        .output()
        .unwrap();
}

fn bspc_set_border_width(node: &Node, width: usize) {
    Command::new("bspc")
        .arg("config")
        .arg("-n")
        .arg(node.0.as_str())
        .arg("border_width")
        .arg(width.to_string())
        .output()
        .unwrap();
}

fn server() {
    let (tx, rx) = channel::<Event>();

    let mut state = State::new();

    subscribe_bspc(BspcSubCommand::NodeAdd, tx.clone());
    subscribe_bspc(BspcSubCommand::NodeRemove, tx.clone());
    subscribe_client(tx);

    for state_change in &rx {
        dbg!(&state_change);
        match state_change {
            Event::AddNode(node) => {
                state.untagged_nodes.insert(node);
            }
            Event::RemoveNode(node) => {
                state.untagged_nodes.remove(&node);
                state.tagged_nodes.remove(&node);
            }
            Event::TagNode(node) => {
                if state.tagged_nodes.contains(&node) {
                    bspc_set_border_width(&node, 1);
                    state.tagged_nodes.remove(&node);
                    state.untagged_nodes.insert(node);
                } else {
                    bspc_set_border_width(&node, 3);
                    state.untagged_nodes.remove(&node);
                    if !state.tag_shown {
                        bspc_toggle_visibility(&node);
                    }
                    state.tagged_nodes.insert(node);
                }
            }
            Event::ShowTag => {
                state.tag_shown = !state.tag_shown;
                for node in &state.tagged_nodes {
                    bspc_toggle_visibility(node);
                }
            }
        };
        dbg!(&state);
    }
}

fn send_client_message(message: &str) {
    let mut stream = UnixStream::connect(SOCKET_PATH).unwrap();
    stream.write_all(message.as_bytes()).unwrap();
}

fn tag() {
    let output = Command::new("bspc")
        .arg("query")
        .arg("-N")
        .arg("focused")
        .arg("-n")
        .output()
        .unwrap();
    let node = std::str::from_utf8(output.stdout.as_slice())
        .unwrap()
        .trim();
    send_client_message(&format!("tag {}", node));
}

fn show() {
    send_client_message("show")
}

fn main() {
    let mut args = env::args().skip(1);
    match args.nth(0).unwrap().as_str() {
        "server" => server(),
        "tag" => tag(),
        "show" => show(),
        _ => panic!("wrong argument"),
    }
}
