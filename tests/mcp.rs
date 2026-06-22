//! Drives the real `webtools mcp` stdio server with JSON-RPC frames that need
//! no network (initialize + tools/list) and checks the responses.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

#[test]
fn mcp_initialize_and_tools_list() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_webtools"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn webtools mcp");

    let requests = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n",
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(requests.as_bytes())
        .unwrap();
    // Dropping stdin (taken above) closes it, so the server loop ends.

    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    child.wait().unwrap();

    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "expected 2 responses, got: {out}");

    let init: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["serverInfo"]["name"], "webtools");
    assert!(init["result"]["capabilities"]["tools"].is_object());

    let list: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(list["id"], 2);
    let tools = list["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"fetch"), "tools: {names:?}");
    assert!(names.contains(&"search"), "tools: {names:?}");
}
