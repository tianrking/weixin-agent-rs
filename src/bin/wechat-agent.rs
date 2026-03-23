use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Stdio;
use std::process::{Command, ExitStatus};
use std::time::Duration;

use sysinfo::{Pid, ProcessesToUpdate, Signal, System};
use wechat_rs_sdk::auth::accounts::{delete_account, list_accounts};
use wechat_rs_sdk::runtime::SpaceAgentRouter;
use wechat_rs_sdk::space::{
    available_agents, clear_space_pid, create_space, delete_space, ensure_space_runtime_dirs, inspect_space,
    list_spaces, load_space, read_space_pid, remove_user_binding, set_space_account, set_user_binding,
    space_log_path, switch_space_agent, write_space_pid,
};
use wechat_rs_sdk::storage::state_dir::resolve_state_dir;
use wechat_rs_sdk::{Bot, LoginOptions, Result, StartOptions, WechatError};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").with_target(false).init();

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    match args[0].as_str() {
        "account" => handle_account(&args[1..]).await,
        "space" => handle_space(&args[1..]).await,
        "agent" => handle_agent(&args[1..]).await,
        "bind" => handle_bind(&args[1..]).await,
        "update" => handle_update(&args[1..]).await,
        "daemon" => handle_daemon(&args[1..]).await,
        "run" => handle_run(&args[1..]).await,
        "login" => {
            let id = Bot::login(LoginOptions::default()).await?;
            println!("login success: {id}");
            Ok(())
        }
        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }
        other => Err(WechatError::Api(format!("unknown command: {other}"))),
    }
}

async fn handle_account(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("login") => {
            let id = Bot::login(LoginOptions::default()).await?;
            println!("account added: {id}");
            Ok(())
        }
        Some("ls") => {
            let accounts = list_accounts();
            if accounts.is_empty() {
                println!("no accounts");
            } else {
                for account in accounts {
                    println!(
                        "{}\ttoken={}\tuser_id={}\tsaved_at={}",
                        account.account_id,
                        if account.has_token { "yes" } else { "no" },
                        account.user_id.unwrap_or_else(|| "-".to_string()),
                        account.saved_at.unwrap_or_else(|| "-".to_string())
                    );
                }
            }
            Ok(())
        }
        Some("rm") => {
            let id = required_arg(args, 1, "account id")?;
            delete_account(id)?;
            println!("account removed: {id}");
            Ok(())
        }
        _ => {
            println!("usage: wechat-agent account <login|ls|rm>");
            Ok(())
        }
    }
}

async fn handle_space(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("create") => {
            let name = required_arg(args, 1, "space name")?;
            let agent = option_value(args, "--agent").unwrap_or_else(|| "codex".to_string());
            let account = option_value(args, "--account");
            let space = create_space(name, &agent, account)?;
            println!(
                "space created: {}\tagent={}\taccount={}",
                space.name,
                space.agent,
                space.account_id.unwrap_or_else(|| "-".to_string())
            );
            Ok(())
        }
        Some("ls") => {
            let spaces = list_spaces()?;
            if spaces.is_empty() {
                println!("no spaces");
            } else {
                for space in spaces {
                    print_space_row(
                        &space.name,
                        &space.agent,
                        space.account_id.as_deref(),
                        space.binding_count,
                    );
                }
            }
            Ok(())
        }
        Some("ps") => {
            let spaces = list_spaces()?;
            if spaces.is_empty() {
                println!("no spaces");
            } else {
                for space in spaces {
                    print_space_row(
                        &space.name,
                        &space.agent,
                        space.account_id.as_deref(),
                        space.binding_count,
                    );
                }
            }
            Ok(())
        }
        Some("inspect") => {
            let name = required_arg(args, 1, "space name")?;
            let space = inspect_space(name)?;
            println!("{}", serde_json::to_string_pretty(&space)?);
            Ok(())
        }
        Some("start") => {
            let name = required_arg(args, 1, "space name")?;
            start_space(name)?;
            Ok(())
        }
        Some("stop") => {
            let name = required_arg(args, 1, "space name")?;
            stop_space(name)?;
            Ok(())
        }
        Some("restart") => {
            let name = required_arg(args, 1, "space name")?;
            restart_space(name)?;
            Ok(())
        }
        Some("logs") => {
            let name = required_arg(args, 1, "space name")?;
            let follow = args.iter().any(|arg| arg == "-f" || arg == "--follow");
            let tail = option_value(args, "--tail")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(80);
            show_logs(name, tail, follow)?;
            Ok(())
        }
        Some("rm") => {
            let name = required_arg(args, 1, "space name")?;
            if is_space_running(name) {
                return Err(WechatError::Api(format!("space is running, stop it first: {name}")));
            }
            delete_space(name)?;
            println!("space removed: {name}");
            Ok(())
        }
        Some("bind-account") => {
            let name = required_arg(args, 1, "space name")?;
            let account = required_arg(args, 2, "account id")?;
            let space = set_space_account(name, Some(account.to_string()))?;
            println!(
                "space account bound: {}\taccount={}",
                space.name,
                space.account_id.unwrap_or_else(|| "-".to_string())
            );
            Ok(())
        }
        Some("unbind-account") => {
            let name = required_arg(args, 1, "space name")?;
            let space = set_space_account(name, None)?;
            println!("space account cleared: {}", space.name);
            Ok(())
        }
        _ => {
            println!("usage: wechat-agent space <create|ls|ps|inspect|start|stop|restart|logs|rm|bind-account|unbind-account>");
            Ok(())
        }
    }
}

async fn handle_agent(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("ls") => {
            for agent in available_agents() {
                println!("{agent}");
            }
            Ok(())
        }
        Some("switch") => {
            let space = required_arg(args, 1, "space name")?;
            let agent = required_arg(args, 2, "agent")?;
            let space = switch_space_agent(space, agent)?;
            println!("space agent switched: {}\tagent={}", space.name, space.agent);
            Ok(())
        }
        _ => {
            println!("usage: wechat-agent agent <ls|switch>");
            Ok(())
        }
    }
}

async fn handle_bind(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("ls") => {
            let space = load_space(required_arg(args, 1, "space name")?)?;
            if space.user_bindings.is_empty() {
                println!("no bindings");
            } else {
                for (user, agent) in space.user_bindings {
                    println!("{user}\t{agent}");
                }
            }
            Ok(())
        }
        Some("set") => {
            let space = required_arg(args, 1, "space name")?;
            let user = required_arg(args, 2, "user id")?;
            let agent = required_arg(args, 3, "agent")?;
            set_user_binding(space, user, agent)?;
            println!("binding set: {space}\t{user}\t{agent}");
            Ok(())
        }
        Some("rm") => {
            let space = required_arg(args, 1, "space name")?;
            let user = required_arg(args, 2, "user id")?;
            remove_user_binding(space, user)?;
            println!("binding removed: {space}\t{user}");
            Ok(())
        }
        _ => {
            println!("usage: wechat-agent bind <ls|set|rm>");
            Ok(())
        }
    }
}

async fn handle_daemon(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("start") => {
            start_daemon()?;
            Ok(())
        }
        Some("status") => {
            let pid = daemon_pid();
            println!(
                "daemon\tpid={}\trunning={}\tlog={}",
                pid.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()),
                if daemon_running() { "yes" } else { "no" },
                daemon_log_path().to_string_lossy()
            );
            Ok(())
        }
        Some("stop") => {
            stop_daemon()?;
            Ok(())
        }
        Some("serve") => daemon_serve().await,
        _ => {
            println!("usage: wechat-agent daemon <start|status|stop>");
            println!("note: daemon is experimental; normal usage does not require it");
            Ok(())
        }
    }
}

async fn handle_update(args: &[String]) -> Result<()> {
    if matches!(args.first().map(String::as_str), Some("-h" | "--help" | "help")) {
        println!("usage: wechat-agent update");
        println!("updates the current git checkout and rebuilds the release binary");
        return Ok(());
    }

    let root = detect_project_root().ok_or_else(|| {
        WechatError::Api("update requires running inside the project checkout".to_string())
    })?;

    if !root.join(".git").exists() {
        return Err(WechatError::Api(format!(
            "update requires a git checkout: {}",
            root.to_string_lossy()
        )));
    }

    println!("update root: {}", root.to_string_lossy());
    run_checked("git", &["pull", "--ff-only"], &root)?;
    run_checked("cargo", &["build", "--release", "--locked"], &root)?;
    println!(
        "update complete: binary at {}",
        root.join("target").join("release").join(exe_name()).to_string_lossy()
    );
    Ok(())
}

async fn handle_run(args: &[String]) -> Result<()> {
    let daemonized = args.iter().any(|arg| arg == "--daemonized");
    let space_name = if args.first().map(String::as_str) == Some("--space") {
        required_arg(args, 1, "space name")?
    } else {
        required_arg(args, 0, "space name")?
    };

    let space = load_space(space_name)?;
    let account_id = space
        .account_id
        .clone()
        .ok_or_else(|| WechatError::Api(format!("space has no bound account: {}", space.name)))?;

    let router = SpaceAgentRouter::new(&space).await?;
    let _guard = SpacePidGuard::acquire(&space.name, daemonized)?;
    println!(
        "running space: {}\tagent={}\taccount={}",
        space.name, space.agent, account_id
    );

    Bot::start(
        router,
        StartOptions {
            account_id: Some(account_id),
        },
    )
    .await
}

struct SpacePidGuard {
    space_name: String,
}

impl SpacePidGuard {
    fn acquire(space_name: &str, daemonized: bool) -> Result<Self> {
        let normalized = space_name.to_string();
        if !daemonized && is_space_running(&normalized) {
            return Err(WechatError::Api(format!("space already running: {normalized}")));
        }
        write_space_pid(&normalized, std::process::id())?;
        Ok(Self { space_name: normalized })
    }
}

impl Drop for SpacePidGuard {
    fn drop(&mut self) {
        let _ = clear_space_pid(&self.space_name);
    }
}

fn start_space(name: &str) -> Result<()> {
    let space = load_space(name)?;
    if space.account_id.is_none() {
        return Err(WechatError::Api(format!("space has no bound account: {}", space.name)));
    }
    if is_space_running(&space.name) {
        println!("space already running: {}", space.name);
        return Ok(());
    }

    ensure_space_runtime_dirs(&space.name)?;
    let log_path = space_log_path(&space.name);
    let log = OpenOptions::new().create(true).append(true).open(&log_path)?;
    let log_err = log.try_clone()?;

    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["run", "--space", &space.name, "--daemonized"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }

    let child = cmd.spawn()?;
    write_space_pid(&space.name, child.id())?;
    println!(
        "space started: {}\tpid={}\tlog={}",
        space.name,
        child.id(),
        log_path.to_string_lossy()
    );
    Ok(())
}

fn stop_space(name: &str) -> Result<()> {
    let normalized = load_space(name)?.name;
    let pid = match read_space_pid(&normalized) {
        Some(pid) => pid,
        None => {
            println!("space not running: {normalized}");
            return Ok(());
        }
    };

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        let _ = process.kill_with(Signal::Term);
        let _ = process.kill();
    }
    clear_space_pid(&normalized)?;
    println!("space stopped: {normalized}");
    Ok(())
}

fn restart_space(name: &str) -> Result<()> {
    if is_space_running(name) {
        stop_space(name)?;
        std::thread::sleep(Duration::from_millis(500));
    }
    start_space(name)
}

fn show_logs(name: &str, tail_lines: usize, follow: bool) -> Result<()> {
    let normalized = load_space(name)?.name;
    let path = space_log_path(&normalized);
    if !path.exists() {
        println!("no logs: {}", path.to_string_lossy());
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)?;
    let lines = content.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(tail_lines);
    for line in &lines[start..] {
        println!("{line}");
    }

    if follow {
        let mut file = OpenOptions::new().read(true).open(&path)?;
        let mut pos = file.seek(SeekFrom::End(0))?;
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let len = file.metadata()?.len();
            if len < pos {
                pos = 0;
            }
            if len == pos {
                continue;
            }
            file.seek(SeekFrom::Start(pos))?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            if !buf.is_empty() {
                print!("{buf}");
            }
            pos = len;
        }
    }

    Ok(())
}

fn is_space_running(name: &str) -> bool {
    let Some(pid) = read_space_pid(name) else {
        return false;
    };
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system.process(Pid::from_u32(pid)).is_some()
}

fn daemon_root() -> PathBuf {
    resolve_state_dir().join("wechat-agent")
}

fn daemon_pid_path() -> PathBuf {
    daemon_root().join("daemon.pid")
}

fn daemon_log_path() -> PathBuf {
    daemon_root().join("daemon.log")
}

fn daemon_pid() -> Option<u32> {
    let raw = std::fs::read_to_string(daemon_pid_path()).ok()?;
    raw.trim().parse::<u32>().ok()
}

fn daemon_running() -> bool {
    let Some(pid) = daemon_pid() else {
        return false;
    };
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system.process(Pid::from_u32(pid)).is_some()
}

fn start_daemon() -> Result<()> {
    if daemon_running() {
        println!("daemon already running: pid={}", daemon_pid().unwrap_or_default());
        return Ok(());
    }

    std::fs::create_dir_all(daemon_root())?;
    let log = OpenOptions::new().create(true).append(true).open(daemon_log_path())?;
    let log_err = log.try_clone()?;
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["daemon", "serve"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }

    let child = cmd.spawn()?;
    std::fs::write(daemon_pid_path(), child.id().to_string())?;
    println!(
        "daemon started: pid={}\tlog={}",
        child.id(),
        daemon_log_path().to_string_lossy()
    );
    Ok(())
}

fn stop_daemon() -> Result<()> {
    let Some(pid) = daemon_pid() else {
        println!("daemon not running");
        return Ok(());
    };
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        let _ = process.kill_with(Signal::Term);
        let _ = process.kill();
    }
    let path = daemon_pid_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    println!("daemon stopped");
    Ok(())
}

async fn daemon_serve() -> Result<()> {
    std::fs::create_dir_all(daemon_root())?;
    std::fs::write(daemon_pid_path(), std::process::id().to_string())?;
    println!("daemon serving");
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}

fn detect_project_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("src").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn run_checked(program: &str, args: &[&str], workdir: &std::path::Path) -> Result<()> {
    let status = Command::new(program).args(args).current_dir(workdir).status()?;
    if status.success() {
        return Ok(());
    }
    Err(command_failed(program, args, status))
}

fn command_failed(program: &str, args: &[&str], status: ExitStatus) -> WechatError {
    WechatError::Api(format!(
        "command failed: {} {} (status: {})",
        program,
        args.join(" "),
        status
    ))
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "wechat-agent.exe"
    } else {
        "wechat-agent"
    }
}

fn required_arg<'a>(args: &'a [String], idx: usize, label: &str) -> Result<&'a str> {
    args.get(idx)
        .map(String::as_str)
        .ok_or_else(|| WechatError::Api(format!("missing {label}")))
}

fn print_help() {
    println!(
        "wechat-agent\n\nUSAGE:\n  wechat-agent <command>\n\nCORE COMMANDS:\n  account login|ls|rm\n  space create|ls|inspect|start|stop|restart|logs|rm|bind-account|unbind-account\n  agent ls|switch\n  bind ls|set|rm\n  update\n\nLOW-LEVEL:\n  run --space <name>\n  daemon start|status|stop  (experimental)\n\nEXAMPLES:\n  wechat-agent account login\n  wechat-agent space create dev --agent codex\n  wechat-agent space bind-account dev my-wechat-bot\n  wechat-agent space start dev\n  wechat-agent space ls\n  wechat-agent space inspect dev\n  wechat-agent space logs dev --tail 100 -f\n  wechat-agent update\n"
    );
}

fn print_space_row(name: &str, agent: &str, account: Option<&str>, bindings: usize) {
    let pid = read_space_pid(name);
    println!(
        "{}\trunning={}\tpid={}\tagent={}\taccount={}\tbindings={}",
        name,
        if is_space_running(name) { "yes" } else { "no" },
        pid.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()),
        agent,
        account.unwrap_or("-"),
        bindings
    );
}
