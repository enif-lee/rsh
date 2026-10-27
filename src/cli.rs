use crate::client::{self, Attached, Input, Outcome, Target, DEFAULT_ESCAPE};
use clap::{Parser, Subcommand};
use crate::crypto::{new_token, Key};
use crate::forward::{Forwarder, Spec};
use crate::net::{self, kill_sessions, list_sessions_wait, AuthStore, HelloReply, HostMux};
use crate::proto::{Action, SessionInfo, TmuxInfo};
use crate::pty::RawTerm;
use crate::server::Server;
use std::env;
use std::fs;
use std::io::{self, BufRead, ErrorKind, Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};


pub const DEFAULT_PORT: u16 = 2222;

pub fn main() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();
    let tty = crate::pty::is_tty(0) && crate::pty::is_tty(1);
    let mut input = Keys::Fd;
    match session(&args, &mut input, &mut io::stdout(), tty) {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(io::stderr(), "rsh: {err}");
            2
        }
    }
}

/// Runs the CLI without a tty: menus read numbered lines, and whatever
/// stdin remains is typed into the session.
pub fn run(args: &[String], stdin: &mut impl BufRead, stdout: &mut impl Write) -> Result<i32, String> {
    let mut input = Keys::Lines(Box::new(stdin));
    session(args, &mut input, stdout, false)
}

enum Keys<'a> {
    /// Unbuffered reads of fd 0, so nothing meant for the session is
    /// swallowed by a menu.
    Fd,
    Lines(Box<dyn BufRead + 'a>),
}

impl Keys<'_> {
    fn byte(&mut self) -> Result<Option<u8>, String> {
        let mut byte = [0u8; 1];
        let n = match self {
            Keys::Fd => {
                let n = unsafe { libc::read(0, byte.as_mut_ptr().cast(), 1) };
                if n < 0 {
                    return Err(io::Error::last_os_error().to_string());
                }
                n as usize
            }
            Keys::Lines(reader) => reader.read(&mut byte).map_err(|err| err.to_string())?,
        };
        Ok(if n == 0 { None } else { Some(byte[0]) })
    }

    fn rest(&mut self) -> Vec<u8> {
        let mut rest = Vec::new();
        if let Keys::Lines(reader) = self {
            let _ = reader.read_to_end(&mut rest);
        }
        rest
    }
}

const AFTER_HELP: &str = "\
rsh <host>
  The host is set up over ssh when there is no token for it, when its
  server does not answer, or when it runs another build than this rsh:
  this build is copied there (a binary for its platform comes from
  ~/.rsh/dist, see scripts/dist.sh), its server is upgraded in place, or
  replaced if it cannot hand over, and the token is saved locally.
  With --peer the server is used as it is.
  When the host has live sessions, a menu offers them and a new session.
  rsh <host> <name> opens or resumes the session called <name>; a new
  session without a name gets the first free remote-<n>. The name and
  host show at the top right (orange while ~ waits for a command).
  rsh <host> <name> --tmux joins the host's tmux session <name> (made if
  missing, first free remote-<n> without a name); detaching in tmux or ~.
  leaves it running in tmux and ends the rsh session.
  rsh ls and ~l list rsh sessions and tmux sessions together.

keys in a session (typed at the start of a line, as in ssh)
  ~.             detach; the session keeps running (in a tmux session:
                 leave tmux; the tmux session keeps running)
  ~l             sessions of every known host; Up/Down and Enter switch
                 to one, q closes
  ~r             repaint the screen from the server
  ~h             these keys
  ~~             send ~
  RSH_ESCAPE_KEY sets another escape character; empty disables it.
  A control character (e.g. RSH_ESCAPE_KEY=$'\\x1e' for Ctrl-^) works
  anywhere, not only at the start of a line.

menus
  Up/Down move, typing filters, Enter selects, Esc cancels.
  rsh ls groups rows under each host, numbered from 1 per host; picking
  a host line goes on as rsh <host> does. Without a tty a line with the
  number selects (<host> <number> when several hosts answer, <host>
  alone for the host line); an empty line takes the first session, q
  cancels.

auth file
  one \"host token [port]\" line per host
";

#[derive(Clone)]
struct Options {
    command: Sub,
    name: Option<String>,
    host: Option<String>,
    target: Option<String>,
    run: Vec<String>,
    /// Tokens this client holds for hosts (`host token [port]`).
    auth: PathBuf,
    /// Tokens this machine's server accepts; never the client's file, so a
    /// token handed out by one host cannot open a shell on another.
    authorized: PathBuf,
    /// Both files are at their defaults, so the old shared file may be split.
    default_files: bool,
    client_name: Option<String>,
    peer: Option<String>,
    bind: Option<SocketAddr>,
    ssh: String,
    /// rsh on the remote host; default `~/.rsh/bin/rsh`.
    server: Option<String>,
    disconnect_timeout: Option<Duration>,
    local_forwards: Vec<Spec>,
    remote_forwards: Vec<Spec>,
    bench: Bench,
    /// Attach this session id (picked in `rsh ls`).
    attach_id: Option<u64>,
    /// Join the tmux session `name` instead of an rsh session.
    tmux: bool,
    /// `--compress`, over `compress` in the config.
    compress: Option<crate::config::Compress>,
}

/// `rsh test` settings.
#[derive(Clone)]
struct Bench {
    size: (u16, u16),
    mb: usize,
    file: Option<PathBuf>,
    seconds: f64,
    /// `rsh test render` frames a second at most, 0 for no limit.
    fps: u16,
    /// Count the frames that reach this terminal (`rsh test render <host>`).
    count_frames: bool,
}

#[derive(Clone, PartialEq, Eq)]
enum Sub {
    HandoffFormat,
    Upgrade,
    Version,
    Connect,
    List,
    Kill,
    Serve,
    Install,
    Bootstrap,
    TestRender,
    TestEmulator,
    TestSession,
}

/// UDP remote shell: sessions stay on the server across disconnects.
#[derive(Parser)]
#[command(name = "rsh", disable_version_flag = true, after_help = AFTER_HELP, override_usage = USAGE)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
    /// Open or resume a shell on this host
    host: Option<String>,
    /// Session to open or resume [default: the only one, a menu, or a new
    /// remote-<n>]
    #[arg(value_name = "NAME", conflicts_with = "name")]
    session: Option<String>,
    /// Run this in a new session; rsh exits with its status
    #[arg(last = true, value_name = "COMMAND")]
    run: Vec<String>,
    /// Session name to create or resume
    #[arg(long)]
    name: Option<String>,
    /// Join the host's tmux session NAME (tmux new-session -A); ~. leaves it
    #[arg(long, conflicts_with = "run")]
    tmux: bool,
    /// Compress the session's output: always (any link), adaptive (slow or
    /// far links), none; over `compress` in ~/.rsh/config.yaml (always)
    #[arg(long, value_name = "MODE", value_parser = parse_compress)]
    compress: Option<crate::config::Compress>,
    /// Forward a local port to host:port as seen from the server
    #[arg(short = 'L', value_name = "[bind:]port:host:port", value_parser = Spec::parse)]
    local: Vec<Spec>,
    /// Forward a server port to host:port as seen from here
    #[arg(short = 'R', value_name = "[bind:]port:host:port", value_parser = Spec::parse)]
    remote: Vec<Spec>,
    /// Tokens this client holds [env: RSH_AUTH] [default: ~/.rsh/known]
    #[arg(long, global = true, value_name = "FILE")]
    auth: Option<PathBuf>,
    /// serve/bootstrap: tokens the server accepts [default: ~/.rsh/authorized]
    #[arg(long, global = true, value_name = "FILE")]
    authorized: Option<PathBuf>,
    /// Server address [default: <host>:2222]
    #[arg(long, global = true, env = "RSH_PEER", value_name = "HOST:PORT")]
    peer: Option<String>,
    /// ssh used to set up a host
    #[arg(long, global = true, default_value = "ssh", value_name = "COMMAND")]
    ssh: String,
    /// rsh on the remote host [default: ~/.rsh/bin/rsh]
    #[arg(long, global = true, value_name = "PATH")]
    server: Option<String>,
    /// serve/bootstrap: listen address [default: 0.0.0.0:2222]
    #[arg(long, global = true, value_name = "IP:PORT")]
    bind: Option<SocketAddr>,
    /// serve: end sessions without a client for this many seconds; by
    /// default detached sessions are kept until they exit
    #[arg(long, global = true, value_name = "SECONDS")]
    disconnect_timeout: Option<u64>,
    /// bootstrap: the client the token is for
    #[arg(long, global = true, hide = true)]
    client: Option<String>,
    /// Print the version
    #[arg(short = 'V', long)]
    version: bool,
    #[arg(long, hide = true)]
    handoff_format: bool,
}

#[derive(Subcommand)]
enum CliCommand {
    /// List sessions on known hosts, then pick one
    Ls { host: Option<String> },
    /// End a session
    Kill {
        host: String,
        /// Session id or name
        target: String,
    },
    /// Run the UDP server in the foreground
    Serve {
        #[command(subcommand)]
        action: Option<ServeAction>,
    },
    /// Start a server if needed and print its token
    Bootstrap,
    /// Measure how fast sessions run
    Test {
        #[command(subcommand)]
        what: TestCommand,
    },
}

#[derive(Subcommand)]
enum TestCommand {
    /// Draw an ASCII-art animation on this terminal as fast as it takes it
    /// and report frames per second and bytes per second; run inside an rsh
    /// session to test the whole way to the terminal
    Render {
        /// Draw on that host, in a session shown here: the frames take the
        /// whole way, and this terminal's count of them is reported too
        host: Option<String>,
        /// How long to draw
        #[arg(long, default_value_t = 5.0, value_parser = parse_seconds)]
        seconds: f64,
        /// Frames a second at most; 0 draws as fast as everything on the
        /// way takes them
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u16).range(0..=1000))]
        fps: u16,
    },
    /// Time this build's terminal emulator on typical output: input
    /// throughput and the repaint an attaching client gets
    Emulator {
        /// Terminal size
        #[arg(long, default_value = "120x40", value_parser = parse_size)]
        size: (u16, u16),
        /// MiB of each generated workload
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u16).range(1..=1024))]
        mb: u16,
        /// Time this recorded output instead (e.g. from `script`)
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,
    },
    /// Time a session on a host already set up: request round trips and
    /// bulk output from a new session
    Session {
        host: String,
        /// MiB of output
        #[arg(long, default_value_t = 16, value_parser = clap::value_parser!(u16).range(1..=4096))]
        mb: u16,
    },
}

fn parse_compress(text: &str) -> Result<crate::config::Compress, String> {
    crate::config::Compress::from_name(text).ok_or(format!("compress must be adaptive, always or none, not `{text}`"))
}

fn parse_seconds(text: &str) -> Result<f64, String> {
    text.parse::<f64>().ok().filter(|s| (0.1..=3600.0).contains(s)).ok_or(format!("seconds must be 0.1 to 3600, not {text}"))
}

fn parse_size(text: &str) -> Result<(u16, u16), String> {
    let (cols, rows) = text.split_once('x').ok_or("size is COLSxROWS")?;
    let parse = |n: &str| n.parse::<u16>().ok().filter(|n| (2..=1000).contains(n)).ok_or(format!("bad size {text}"));
    Ok((parse(cols)?, parse(rows)?))
}

#[derive(Subcommand)]
enum ServeAction {
    /// Start the server when this user logs in (macOS); if one is running,
    /// upgrade it in place instead
    Install,
    /// Make the running server exec the binary at its path again; sessions
    /// and clients carry on
    Upgrade,
}

const USAGE: &str = "rsh [OPTIONS] <HOST> [NAME] [-- <COMMAND>...]
       rsh <ls|kill|serve|bootstrap|test> ...";

impl Options {
    fn parse(args: &[String]) -> Result<Self, clap::Error> {
        let cli = Cli::try_parse_from(std::iter::once("rsh".to_string()).chain(args.iter().cloned()))?;
        // `rsh <host> serve` names a session `serve`; clap would read the
        // word as the subcommand.
        if let (Some(_), Some(host)) = (&cli.command, &cli.host) {
            if let Some(at) = args.iter().position(|arg| arg == host).filter(|at| at + 1 < args.len()) {
                let mut named = args.to_vec();
                named[at + 1] = format!("--name={}", named[at + 1]);
                return Self::parse(&named);
            }
        }
        // RSH_AUTH as RSH_PEER: an empty value means unset.
        let auth = cli.auth.or_else(|| env::var_os("RSH_AUTH").filter(|v| !v.is_empty()).map(PathBuf::from));
        let bench = match &cli.command {
            Some(CliCommand::Test { what: TestCommand::Emulator { size, mb, file } }) => {
                Bench { size: *size, mb: usize::from(*mb), file: file.clone(), seconds: 0.0, fps: 0, count_frames: false }
            }
            Some(CliCommand::Test { what: TestCommand::Render { seconds, fps, .. } }) => {
                Bench { size: (0, 0), mb: 0, file: None, seconds: *seconds, fps: *fps, count_frames: false }
            }
            Some(CliCommand::Test { what: TestCommand::Session { mb, .. } }) => {
                Bench { size: (120, 40), mb: usize::from(*mb), file: None, seconds: 0.0, fps: 0, count_frames: false }
            }
            _ => Bench { size: (120, 40), mb: 0, file: None, seconds: 0.0, fps: 0, count_frames: false },
        };
        let (command, host, target) = match cli.command {
            _ if cli.version => (Sub::Version, None, None),
            _ if cli.handoff_format => (Sub::HandoffFormat, None, None),
            None => (Sub::Connect, cli.host, None),
            Some(CliCommand::Ls { host }) => (Sub::List, host, None),
            Some(CliCommand::Kill { host, target }) => (Sub::Kill, Some(host), Some(target)),
            Some(CliCommand::Serve { action: None }) => (Sub::Serve, None, None),
            Some(CliCommand::Serve { action: Some(ServeAction::Install) }) => (Sub::Install, None, None),
            Some(CliCommand::Serve { action: Some(ServeAction::Upgrade) }) => (Sub::Upgrade, None, None),
            Some(CliCommand::Bootstrap) => (Sub::Bootstrap, None, None),
            Some(CliCommand::Test { what: TestCommand::Render { ref host, .. } }) => (Sub::TestRender, host.clone(), None),
            Some(CliCommand::Test { what: TestCommand::Emulator { .. } }) => (Sub::TestEmulator, None, None),
            Some(CliCommand::Test { what: TestCommand::Session { ref host, .. } }) => (Sub::TestSession, Some(host.clone()), None),
        };

        Ok(Options {
            command,
            name: cli.name.or(cli.session),
            host,
            target,
            run: cli.run,
            default_files: auth.is_none() && cli.authorized.is_none(),
            auth: auth.unwrap_or_else(|| home().join(".rsh").join("known")),
            authorized: cli.authorized.unwrap_or_else(|| home().join(".rsh").join("authorized")),
            client_name: cli.client,
            peer: cli.peer.filter(|p| !p.is_empty()),
            bind: cli.bind,
            ssh: cli.ssh,
            server: cli.server,
            disconnect_timeout: cli.disconnect_timeout.map(Duration::from_secs),
            local_forwards: cli.local,
            remote_forwards: cli.remote,
            bench,
            attach_id: None,
            tmux: cli.tmux,
            compress: cli.compress,
        })
    }
}

fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn session(args: &[String], keys: &mut Keys, stdout: &mut dyn Write, tty: bool) -> Result<i32, String> {
    let opts = match Options::parse(args) {
        Ok(opts) => opts,
        Err(err) if matches!(err.kind(), clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand) => {
            write!(stdout, "{}", err.render()).map_err(|err| err.to_string())?;
            return Ok(0);
        }
        Err(err) => return Err(err.render().to_string().trim_start_matches("error: ").trim_end().to_string()),
    };
    migrate_legacy(&opts);
    match opts.command {
        Sub::Serve => run_serve(&opts, stdout),
        Sub::Upgrade => upgrade_server(&opts, stdout).map(|_| 0),
        Sub::HandoffFormat => {
            writeln!(stdout, "rsh-handoff {}", crate::server::HANDOFF_FORMAT).map_err(|err| err.to_string())?;
            Ok(0)
        }
        Sub::Version => {
            writeln!(stdout, "{}", crate::version::describe()).map_err(|err| err.to_string())?;
            Ok(0)
        }
        Sub::Install => install_launch_agent(&opts, stdout).map(|_| 0),
        Sub::Bootstrap => bootstrap(&opts, stdout).map(|_| 0),
        Sub::List => {
            crate::release::upgrade_and_rerun(&home(), args);
            list_and_attach(&opts, keys, stdout, tty)
        }
        Sub::Kill => kill(&opts, stdout),
        Sub::TestRender => match opts.host.clone() {
            None => crate::bench::live(opts.bench.seconds, f64::from(opts.bench.fps), stdout).map(|_| 0).map_err(|err| err.to_string()),
            // A command session on the host, set up and kept at this build
            // like any connect, whose frames this client counts.
            Some(host) => {
                let mut test = opts.clone();
                let bin = crate::remote::remote_bin(opts.server.as_deref());
                test.run = vec![format!("exec {bin} test render --seconds {} --fps {}", opts.bench.seconds, opts.bench.fps)];
                test.name = None;
                test.bench.count_frames = true;
                connect_host(&host, &test, keys, stdout, tty)
            }
        },
        Sub::TestEmulator => crate::bench::emulator(opts.bench.size, opts.bench.mb, opts.bench.file.as_deref(), stdout)
            .map(|_| 0)
            .map_err(|err| err.to_string()),
        Sub::TestSession => {
            let host = opts.host.clone().unwrap_or_default();
            let store = load_store(&opts.auth)?;
            let token = store.token(&host).ok_or("no token for this host; connect once with rsh <host>")?;
            let peer = peer_for(&host, &store, &opts)?;
            crate::bench::session(&host, peer, &Key::from_token(token), opts.bench.mb, stdout).map(|_| 0)
        }
        Sub::Connect => {
            let host = opts.host.clone().ok_or("host required; see rsh --help")?;
            crate::release::upgrade_and_rerun(&home(), args);
            connect_host(&host, &opts, keys, stdout, tty)
        }
    }
}

fn resolve_peer(text: &str) -> Result<SocketAddr, String> {
    let addrs: Vec<SocketAddr> = text
        .to_socket_addrs()
        .map_err(|err| format!("cannot resolve {text}: {err}"))?
        .collect();
    addrs
        .iter()
        .copied()
        .find(SocketAddr::is_ipv4)
        .or_else(|| addrs.first().copied())
        .ok_or_else(|| format!("cannot resolve {text}"))
}

fn ssh_host(host: &str) -> &str {
    host.rsplit('@').next().unwrap_or(host)
}

fn peer_for(host: &str, store: &AuthStore, opts: &Options) -> Result<SocketAddr, String> {
    peer_of(host, store, &opts.ssh, opts.peer.as_deref())
}

fn peer_of(host: &str, store: &AuthStore, ssh: &str, peer: Option<&str>) -> Result<SocketAddr, String> {
    if let Some(peer) = peer {
        return resolve_peer(peer);
    }
    let port = store.port(host).unwrap_or(DEFAULT_PORT);
    // The same address ssh reaches, so a `Host` alias with a `HostName`
    // in ~/.ssh/config works for the UDP session too.
    let name = crate::remote::Ssh::new(ssh, host, None)
        .hostname()
        .unwrap_or_else(|| ssh_host(host).to_string());
    resolve_peer(&format!("{name}:{port}"))
}

/// Moves the client tokens of the old `~/.rsh/auth`, which held both kinds,
/// into `known`.
fn migrate_legacy(opts: &Options) {
    let legacy = home().join(".rsh").join("auth");
    if !opts.default_files || !legacy.exists() {
        return;
    }
    let Ok(old) = AuthStore::load_any(&legacy) else {
        return;
    };
    let mut known = AuthStore::load(&opts.auth).unwrap_or_default();
    // The server's old shared token is dropped, not carried over: clients
    // that held it are set up again over ssh and get a token of their own.
    for (name, token) in old.entries() {
        let port = old.port(&name);
        if name != "rsh-bootstrap" && known.token(&name).is_none() {
            known.insert_with_port(name, token, port);
        }
    }
    if known.save(&opts.auth).is_ok() {
        let _ = fs::rename(&legacy, legacy.with_extension("migrated"));
        let _ = writeln!(
            io::stderr(),
            "rsh: moved the host tokens of ~/.rsh/auth to {}; its server token is no longer accepted",
            opts.auth.display()
        );
    }
}

fn load_store(path: &Path) -> Result<AuthStore, String> {
    match AuthStore::load(path) {
        Ok(store) => Ok(store),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(AuthStore::new()),
        Err(err) => Err(format!("{}: {err}", path.display())),
    }
}

fn run_serve(opts: &Options, stdout: &mut dyn Write) -> Result<i32, String> {
    let store = load_store(&opts.authorized)?;
    let bind = opts.bind.unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT)));
    let exe = env::current_exe().map_err(|err| err.to_string())?;
    crate::upgrade::install_handler();
    crate::upgrade::unblock_request();
    let (server, resumed) = Server::serve_process(bind, &store, opts.authorized.clone(), opts.disconnect_timeout, exe.clone())
        .map_err(|err| format!("bind {bind}: {err}"))?;
    let pidfile = crate::upgrade::pidfile(&opts.authorized, server.addr().port());
    if let Err(err) = crate::upgrade::write_pidfile(&pidfile, &exe) {
        let _ = writeln!(io::stderr(), "rsh: {}: {err}", pidfile.display());
    }
    let version = env!("CARGO_PKG_VERSION");
    if resumed > 0 {
        writeln!(stdout, "rsh {version} serving {} (upgraded, kept {resumed} sessions)", server.addr())
    } else {
        writeln!(stdout, "rsh {version} serving {}", server.addr())
    }
    .map_err(|err| err.to_string())?;
    stdout.flush().map_err(|err| err.to_string())?;
    loop {
        thread::sleep(Duration::from_secs(24 * 60 * 60));
    }
}

/// Run over ssh by a client during onboarding: makes sure a token exists
/// and that a server of this very build answers on the port, upgrading a
/// running server in place when it can hand over its sessions and replacing
/// it otherwise, then prints `RSH CONNECT <port> <token>`. Diagnostics go
/// to stderr; stdout carries only that line.
fn bootstrap(opts: &Options, stdout: &mut dyn Write) -> Result<(), String> {
    let _lock = BootstrapLock::take(&opts.authorized)?;
    let mut store = load_store(&opts.authorized)?;
    // One token per client, so one can be revoked by deleting its line.
    let client: String = opts
        .client_name
        .as_deref()
        .unwrap_or("manual")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "@._-".contains(*c))
        .take(64)
        .collect();
    let name = format!("client:{client}");
    let name = name.as_str();
    let (token, fresh) = match store.token(name) {
        Some(token) => (token.to_string(), false),
        None => {
            let token = new_token();
            store.insert(name, token.clone());
            store.save(&opts.authorized).map_err(|err| err.to_string())?;
            (token, true)
        }
    };
    let bind = opts.bind.unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT)));
    let local = SocketAddr::from(([127, 0, 0, 1], bind.port()));
    let key = Key::from_token(&token);
    let mut log = io::stderr();
    // A running server re-reads the auth file every two seconds, so a token
    // written just now needs a little longer before it is known.
    let wait = if fresh { Duration::from_millis(4500) } else { Duration::from_millis(1500) };
    let server = net::list_with_server(local, &key, wait).ok().map(|(_, line)| line);
    match server.as_deref().map(crate::version::parse) {
        Some(Some(remote)) if crate::version::need(Some(&remote)) == crate::version::Need::Nothing => {}
        Some(found) => {
            // Another build answers. The upgrade that keeps sessions works
            // when it reads our handoff format and runs from this binary's
            // path; it is checked again afterwards.
            let running = crate::upgrade::running(&crate::upgrade::pidfile(&opts.authorized, bind.port()));
            let same_path = running.as_ref().is_some_and(|server| {
                env::current_exe().ok().and_then(|me| fs::canonicalize(me).ok()) == fs::canonicalize(&server.exe).ok()
            });
            let handoff = found.as_ref().and_then(|r| r.handoff);
            let mut upgraded = false;
            if same_path && handoff.is_none_or(|h| h == crate::server::HANDOFF_FORMAT) {
                let mut quiet = Vec::new();
                upgraded = upgrade_server(opts, &mut quiet).is_ok()
                    && net::list_with_server(local, &key, Duration::from_secs(2))
                        .ok()
                        .and_then(|(_, line)| crate::version::parse(&line))
                        .is_some_and(|r| crate::version::need(Some(&r)) == crate::version::Need::Nothing);
            }
            if upgraded {
                let _ = writeln!(log, "rsh: server upgraded in place; sessions kept");
            } else {
                let _ = writeln!(log, "rsh: replacing the rsh server on port {} (its sessions end)", bind.port());
                stop_servers(opts, bind.port(), true)?;
                start_server(opts, bind, &key)?;
            }
        }
        None => {
            // Nothing answers with our token: an older rsh, or none, may hold
            // the port. A server of this protocol is left alone.
            stop_servers(opts, bind.port(), false)?;
            start_server(opts, bind, &key)?;
        }
    }
    if let Ok(exe) = env::current_exe() {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        if let Some(note) = link_on_path(&home(), &exe, &shell) {
            writeln!(stdout, "RSH NOTE {note}").map_err(|err| err.to_string())?;
        }
    }
    writeln!(stdout, "RSH CONNECT {} {token}", bind.port()).map_err(|err| err.to_string())?;
    Ok(())
}

/// Makes `rsh` a command in the user's shells here: a link in
/// `~/.local/bin`, which most login profiles put on PATH. Another program
/// already there is left alone. Returns a note when the user's login shell
/// still would not find this rsh.
fn link_on_path(home: &Path, exe: &Path, shell: &str) -> Option<String> {
    let dir = home.join(".local").join("bin");
    let link = dir.join("rsh");
    let real = fs::canonicalize(exe).ok()?;
    // Only the copy rsh manages (`~/.rsh/bin`), not a build run from elsewhere.
    if !real.starts_with(fs::canonicalize(home.join(".rsh")).ok()?) {
        return None;
    }
    let current = fs::canonicalize(&link).ok();
    if current.as_ref() != Some(&real) {
        match fs::symlink_metadata(&link) {
            // A link to nothing, e.g. a removed install, is replaced.
            Ok(meta) if meta.file_type().is_symlink() && current.is_none() => {
                let _ = fs::remove_file(&link);
            }
            Ok(_) => return Some(format!("{} is another program; this rsh is {}", link.display(), exe.display())),
            Err(_) => {}
        }
        if fs::create_dir_all(&dir).and_then(|_| std::os::unix::fs::symlink(exe, &link)).is_err() {
            return Some(format!("could not link {}; this rsh is {}", link.display(), exe.display()));
        }
    }
    // What a fresh login shell has on PATH, with a few seconds for its rc
    // files: started clean (only the login variables), in a session of its
    // own so it cannot take a terminal, and killed with all it started.
    let mut command = Command::new(shell);
    command
        .args(["-lic", "printf '\\nRSH-PATH:%s\\n' \"$PATH\""])
        .env_clear()
        .env("HOME", home)
        .env("SHELL", shell)
        .env("TERM", "dumb")
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for name in ["USER", "LOGNAME"] {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    let mut child = command.spawn().ok()?;
    let stdout = child.stdout.take()?;
    let (send, receive) = std::sync::mpsc::channel();
    // Read on a thread up to the marker line: a background job of the rc
    // files may keep the pipe open after the shell has answered.
    thread::spawn(move || {
        for line in io::BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(path) = line.strip_prefix("RSH-PATH:") {
                let _ = send.send(path.to_string());
                return;
            }
        }
    });
    let text = receive.recv_timeout(Duration::from_secs(3)).ok();
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.wait();
    let path = text?;
    let real_dir = fs::canonicalize(&dir).ok();
    if path.split(':').any(|entry| fs::canonicalize(entry).ok() == real_dir) {
        return None;
    }
    Some(format!(
        "{} is not on PATH for {shell}; to run rsh there, add: export PATH=\"$HOME/.local/bin:$PATH\"",
        dir.display()
    ))
}

/// Serialises `rsh bootstrap` runs of one user, so two clients onboarding
/// at once do not replace each other's server or token.
struct BootstrapLock(PathBuf);

impl BootstrapLock {
    fn take(auth: &Path) -> Result<Self, String> {
        let dir = auth.parent().unwrap_or(Path::new(".")).join(".bootstrap.lock");
        if let Some(parent) = dir.parent() {
            let _ = net::private_dir(parent);
        }
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match fs::create_dir(&dir) {
                Ok(()) => return Ok(Self(dir)),
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                    let stale = fs::metadata(&dir)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|age| age > Duration::from_secs(60));
                    if stale {
                        let _ = fs::remove_dir(&dir);
                    } else if Instant::now() > deadline {
                        return Err(format!("another rsh bootstrap holds {}", dir.display()));
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(err) => return Err(format!("{}: {err}", dir.display())),
            }
        }
    }
}

impl Drop for BootstrapLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

/// Ends this user's `rsh serve` processes on `port`. A server whose binary
/// speaks this protocol is kept unless `same_protocol_too` (it may use
/// another auth file, and is not ours to replace); then this is an error.
/// SIGTERM, then SIGKILL after two seconds for those still there.
fn stop_servers(opts: &Options, port: u16, same_protocol_too: bool) -> Result<(), String> {
    let me = std::process::id() as i32;
    let mut pids = Vec::new();
    for pid in crate::upgrade::own_pids() {
        let Some(args) = crate::upgrade::process_args(pid) else {
            continue;
        };
        if pid == me || !crate::upgrade::is_rsh_serve(&args) || crate::upgrade::serve_port(&args, DEFAULT_PORT) != port {
            continue;
        }
        let compatible = crate::upgrade::process_version(pid, &args)
            .and_then(|line| crate::version::parse(&line))
            .is_some_and(|r| r.protocol == Some(crate::version::PROTOCOL));
        if compatible && !same_protocol_too {
            return Err(format!(
                "port {port} is served by an rsh of this protocol (pid {pid}) that does not know this token; \
                 it may use another auth file than {}",
                opts.authorized.display()
            ));
        }
        pids.push(pid);
    }
    for &pid in &pids {
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
    }
    let gone = |pid: &i32| unsafe { libc::kill(*pid, 0) } != 0;
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline && !pids.iter().all(gone) {
        thread::sleep(Duration::from_millis(50));
    }
    for pid in pids.iter().filter(|pid| !gone(pid)) {
        unsafe {
            libc::kill(*pid, libc::SIGKILL);
        }
    }
    Ok(())
}

/// Starts this binary as `rsh serve` in its own session and waits until it
/// answers, so the ssh that ran `bootstrap` can end.
fn start_server(opts: &Options, bind: SocketAddr, key: &Key) -> Result<(), String> {
    let local = SocketAddr::from(([127, 0, 0, 1], bind.port()));
    let deadline = Instant::now() + Duration::from_secs(3);
    while std::net::UdpSocket::bind(bind).is_err() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    let exe = env::current_exe().map_err(|err| err.to_string())?;
    let log_path = home().join(".rsh").join("serve.log");
    let _ = net::private_dir(&home().join(".rsh"));
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|err| err.to_string())?;
    let mut command = Command::new(exe);
    command
        .args(["serve", "--bind", &bind.to_string(), "--authorized"])
        .arg(&opts.authorized)
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|err| err.to_string())?)
        .stderr(log);
    if let Some(limit) = opts.disconnect_timeout {
        command.args(["--disconnect-timeout", &limit.as_secs().to_string()]);
    }
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    command.spawn().map_err(|err| format!("start server: {err}"))?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while list_sessions_wait(local, key, Duration::from_millis(300)).is_err() {
        if Instant::now() > deadline {
            return Err(format!("server did not start; see {}", log_path.display()));
        }
    }
    Ok(())
}

/// Brings the host to this client's build over ssh and saves the token its
/// server prints: installs or replaces rsh there when it is missing or
/// another build, and starts or upgrades the server.
fn onboard(host: &str, opts: &Options, store: &mut AuthStore, why: &str) -> Result<(), String> {
    let mut err = io::stderr();
    let ssh = crate::remote::Ssh::new(&opts.ssh, host, crate::remote::control_dir());
    let bin = crate::remote::remote_bin(opts.server.as_deref());
    let _ = writeln!(err, "rsh: {why}; setting up {host} over ssh");
    let probe = crate::remote::probe(&ssh, &bin).map_err(|e| format!("{host}: {e}"))?;
    let binary = crate::remote::plan(&probe, &home()).map_err(|e| format!("{host}: {e}"))?;
    if let Some(binary) = &binary {
        let what = match &probe.installed {
            None => "installing".to_string(),
            Some(old) => format!("replacing rsh {} {}", old.version, old.build.as_deref().unwrap_or("(old build)")),
        };
        let _ = writeln!(
            err,
            "rsh: {what} with rsh {} {} on {host} ({}, {} KiB)",
            crate::version::VERSION,
            crate::version::build(),
            probe.platform,
            binary.bytes.len() / 1024
        );
    }
    let (port, token) =
        crate::remote::install_and_bootstrap(&ssh, &bin, binary.as_ref()).map_err(|e| format!("{host}: {e}"))?;
    store.insert_with_port(host, token, (port != DEFAULT_PORT).then_some(port));
    store.save(&opts.auth).map_err(|err| err.to_string())
}

/// Whether onboarding could bring a server reporting `server` to this
/// build without ssh telling us more: false when it names a platform for
/// which no binary of this build is at hand, so connecting does not run
/// ssh every time only to find that out again.
fn can_update(server: &crate::version::Remote) -> bool {
    match &server.platform {
        Some(platform) => crate::remote::binary_for(platform, &home()).is_ok_and(|binary| binary.this_build),
        None => true,
    }
}

fn connect_host(host: &str, opts: &Options, keys: &mut Keys, stdout: &mut dyn Write, tty: bool) -> Result<i32, String> {
    // Before anything is done on the host: a setting meant to be off is not
    // found wrong only after onboarding.
    let config = crate::config::Config::load(&home().join(".rsh").join("config.yaml"))?;
    let mut store = load_store(&opts.auth)?;
    // With an explicit --peer the user manages the server; without one the
    // host is set up and kept at this client's build over ssh, at most once
    // per connect.
    let managed = opts.peer.is_none();
    let mut onboarded = false;
    let tmux_name = opts.name.as_deref().map(crate::tmux::clean_name).filter(|name| !name.is_empty());
    if store.token(host).is_none() {
        if !managed {
            return Err("authentication required: no token for this host".to_string());
        }
        onboard(host, opts, &mut store, "no token for this host")?;
        onboarded = true;
    }
    let (key, peer, listing) = loop {
        let token = store.token(host).ok_or("authentication required")?.to_string();
        let key = Key::from_token(&token);
        let peer = peer_for(host, &store, opts)?;
        // tmux sessions only to name a new one.
        let tmux = opts.tmux && tmux_name.is_none();
        match net::list(peer, &key, Duration::from_secs(3), tmux) {
            Ok(listing) => {
                let server = listing.server.clone();
                let remote = crate::version::parse(&server);
                let need = crate::version::need(remote.as_ref());
                if managed && !onboarded && need != crate::version::Need::Nothing {
                    if need == crate::version::Need::Update && remote.as_ref().is_some_and(|r| !can_update(r)) {
                        let _ = writeln!(
                            io::stderr(),
                            "rsh: {host} runs {server}; no {} binary of this build to update it with (scripts/dist.sh)",
                            remote.and_then(|r| r.platform).unwrap_or_default()
                        );
                        break (key, peer, listing);
                    }
                    let why = if server.is_empty() { "the server is an older rsh".to_string() } else { format!("the server runs {server}") };
                    onboarded = true;
                    match onboard(host, opts, &mut store, &why) {
                        Ok(()) => continue,
                        // A compatible server still works; say why it stays.
                        Err(e) if need == crate::version::Need::Update => {
                            let _ = writeln!(io::stderr(), "rsh: could not update {host}: {e}");
                        }
                        Err(e) => return Err(e),
                    }
                }
                break (key, peer, listing);
            }
            Err(e) if e.kind() == ErrorKind::TimedOut && managed && !onboarded => {
                onboarded = true;
                onboard(host, opts, &mut store, "the server does not answer")?;
            }
            Err(e) => return Err(format!("{host}: {e}")),
        }
    };
    // The session picked in `rsh ls` may be gone if updating the host could
    // not keep sessions; then it goes on as `rsh <host>` would.
    let sessions = listing.sessions;
    let picked = opts.attach_id.filter(|id| {
        let there = sessions.iter().any(|s| s.id == *id && s.alive);
        if !there {
            let _ = write!(io::stderr(), "rsh: session {id} is no longer on {host}{}", eol(tty));
        }
        there
    });
    let action = if let Some(id) = picked {
        Action::Attach(id)
    } else if opts.tmux {
        Action::Named(tmux_name.unwrap_or_else(|| crate::proto::free_name(|name| listing.tmux.iter().any(|t| t.name == name))))
    } else if !opts.run.is_empty() {
        opts.name.clone().map_or(Action::Default, Action::Named)
    } else if let Some(name) = &opts.name {
        Action::Named(name.clone())
    } else {
        // Sessions joined to tmux sessions are reached as tmux sessions.
        let live: Vec<SessionInfo> = sessions.into_iter().filter(|s| s.host == host && s.alive && !crate::tmux::is_wrapper(&s.command)).collect();
        if live.is_empty() {
            Action::Default
        } else if tty || live.len() > 1 {
            let mut rows: Vec<Row> = live.iter().map(|s| Row::Session(s.clone())).collect();
            rows.push(Row::New);
            write!(stdout, "{host}{}", eol(tty)).map_err(|err| err.to_string())?;
            match choose_row(&rows, keys, stdout, tty)?.map(|index| &rows[index]) {
                Some(Row::Session(s)) => Action::Attach(s.id),
                Some(Row::New) => Action::Named(new_name(&live, tty)?),
                Some(_) => return Err("cancelled".to_string()),
                None => return Ok(0),
            }
        } else {
            Action::Attach(live[0].id)
        }
    };
    let raw = if tty { Some(RawTerm::enter(0).map_err(|err| err.to_string())?) } else { None };
    attach(host, peer, key, action, opts, &config, keys, stdout, raw)
}

/// The command that attaches this session again: the name as an argument,
/// or `--name=` when it would read as a subcommand or an option.
fn resume_command(host: &str, label: &str, tmux: bool) -> String {
    let quoted = crate::remote::shell_quote(label);
    let tmux = if tmux { " --tmux" } else { "" };
    if label.starts_with('-') || ["ls", "kill", "serve", "bootstrap", "test", "help"].contains(&label) {
        format!("rsh {host} --name={quoted}{tmux}")
    } else {
        format!("rsh {host} {quoted}{tmux}")
    }
}

fn eol(tty: bool) -> &'static str {
    if tty {
        "\r\n"
    } else {
        "\n"
    }
}

#[allow(clippy::too_many_arguments)]
fn attach(
    host: &str,
    peer: SocketAddr,
    key: Key,
    action: Action,
    opts: &Options,
    config: &crate::config::Config,
    keys: &mut Keys,
    stdout: &mut dyn Write,
    raw: Option<RawTerm>,
) -> Result<i32, String> {
    let tty = raw.is_some();
    let nl = eol(tty);
    let mut forwarder = Forwarder::new(true);
    for spec in &opts.local_forwards {
        if let Err(err) = forwarder.listen(spec) {
            let _ = write!(io::stderr(), "rsh: local forward {}:{} -> {}: {err}{nl}", spec.bind, spec.port, spec.target);
        }
    }
    if tty {
        client::install_signal_handlers();
    }
    let escape = match env::var("RSH_ESCAPE_KEY") {
        Ok(value) if value.is_empty() => None,
        Ok(value) if value.len() == 1 && value.as_bytes()[0] < 0x80 => Some(value.as_bytes()[0]),
        _ => Some(DEFAULT_ESCAPE),
    };
    // While a session owns the terminal no spinner draws on it (`~l`
    // lists hosts on other threads; a switch says hello over the screen).
    struct Quiet;
    impl Drop for Quiet {
        fn drop(&mut self) {
            net::quiet(false);
        }
    }
    net::quiet(tty);
    let _quiet = Quiet;
    // Picking another session in `~l` attaches it in place of this one;
    // the forwards and the command belong to the first.
    let (mut host, mut peer, mut key, mut action) = (host.to_string(), peer, key, action);
    // The tmux session joined, when the target is one.
    let mut tmux = match &action {
        Action::Named(name) if opts.tmux => Some(name.clone()),
        _ => None,
    };
    let mut forwarder = Some(forwarder);
    let mut first = true;
    // The session left for another, should attaching that one fail.
    let mut left: Option<String> = None;
    let failed = |left: &Option<String>, err: String| match left {
        Some(left) => format!("{err}; detached {left}"),
        None => err,
    };
    loop {
        let size = if tty { crate::pty::window_size(1) } else { (80, 24) };
        let mut mux = HostMux::new();
        let sock = mux.socket_for(&host, peer).map_err(|err| err.to_string())?;
        let remote: Vec<String> = if first { opts.remote_forwards.iter().map(Spec::encode).collect() } else { Vec::new() };
        let run = match &tmux {
            Some(name) => vec![crate::tmux::command(name)],
            None if first => opts.run.clone(),
            None => Vec::new(),
        };
        let mut hello = net::new_hello_forwarding(&host, action.clone(), size, &run, &remote);
        if tty {
            hello.features |= crate::proto::FEATURE_SCREENS;
        }
        match opts.compress.unwrap_or(config.compress) {
            crate::config::Compress::Adaptive => {}
            crate::config::Compress::Always => hello.features |= crate::proto::FEATURE_COMPRESS,
            crate::config::Compress::Off => hello.features &= !(crate::proto::FEATURE_DEFLATE | crate::proto::FEATURE_ZSTREAM),
        }
        let done = match net::hello(&sock, peer, &key, &hello).map_err(|err| failed(&left, format!("{host}: {err}")))? {
            HelloReply::Ready(done) => *done,
            HelloReply::Denied(reason) => return Err(failed(&left, format!("{host}: {}", printable(&reason)))),
        };
        for note in &done.attached.notes {
            let _ = write!(io::stderr(), "rsh: {}{nl}", printable(note));
        }
        let attached = Attached::new(&host, peer, &key, done);
        let input = match keys {
            _ if tty => Input::Tty(0),
            Keys::Fd => Input::Pipe(0),
            Keys::Lines(_) => Input::Script(keys.rest()),
        };
        let frames = (first && opts.bench.count_frames).then(|| std::sync::Arc::new(std::sync::Mutex::new(crate::bench::FrameCount::default())));
        let lister: client::Lister = {
            let store = load_store(&opts.auth).unwrap_or_default();
            let here = match &tmux {
                Some(name) => Target::Tmux(host.clone(), name.clone()),
                None => Target::Rsh(host.clone(), attached.session),
            };
            let (ssh, peer, host) = (opts.ssh.clone(), opts.peer.clone(), host.clone());
            std::sync::Arc::new(move || session_overview(&store, &ssh, peer.as_deref(), &host, &here))
        };
        let outcome = client::run(
            &mut mux,
            &attached,
            input,
            stdout,
            client::Options {
                size,
                escape,
                raw: raw.as_ref(),
                forwarder: forwarder.take(),
                allowed_targets: if first { opts.remote_forwards.iter().map(|spec| spec.target.clone()).collect() } else { Vec::new() },
                sessions: Some(lister),
                frames: frames.clone(),
                tmux: tmux.clone(),
                clipboard_read: config.clipboard_read,
            },
        )
        .map_err(|err| err.to_string())?;
        first = false;
        let label = &printable(&attached.label);
        let resume = resume_command(&host, label, tmux.is_some());
        let (code, message) = match outcome {
            Outcome::Switch(target) => {
                left = Some(format!("{host}:{label}; resume: {resume}"));
                let to = match &target {
                    Target::Rsh(to, _) | Target::Tmux(to, _) => to.clone(),
                };
                let store = load_store(&opts.auth).map_err(|err| failed(&left, err))?;
                let token = store.token(&to).ok_or_else(|| failed(&left, format!("{to}: no token")))?;
                key = Key::from_token(token);
                peer = if to == host { peer } else { peer_for(&to, &store, opts).map_err(|err| failed(&left, err))? };
                host = to;
                (action, tmux) = match target {
                    Target::Rsh(_, id) => (Action::Attach(id), None),
                    Target::Tmux(_, name) => (Action::Named(name.clone()), Some(name)),
                };
                continue;
            }
            // Ending the rsh session hangs up its tmux client: tmux detaches it.
            Outcome::Left => match net::kill_sessions(peer, &key, &host, &attached.session.to_string()) {
                Ok(_) => (0, format!("[rsh: left tmux session {label} on {host}]")),
                Err(err) => (1, format!("[rsh: detached {host}:{label}, but could not end its rsh session: {err}]")),
            },
            Outcome::Detached => (0, format!("[rsh: detached {host}:{label}; resume: {resume}]")),
            // Detached in tmux, or its last shell ended.
            Outcome::Exited(0) if tmux.is_some() => (0, format!("[rsh: tmux client of {label} on {host} ended]")),
            Outcome::Exited(127) if tmux.is_some() => (127, format!("[rsh: tmux not found on {host}]")),
            Outcome::Exited(0) => (0, format!("[rsh: {host}:{label} ended]")),
            Outcome::Exited(code) => (code, format!("[rsh: {host}:{label} ended with status {code}]")),
            Outcome::Kicked(reason) => (1, format!("[rsh: {host}:{label}: {}]", printable(&reason))),
        };
        drop(raw);
        if tty {
            let _ = writeln!(io::stderr(), "\r\n{message}");
        }
        if let Some(count) = frames.and_then(|count| count.lock().ok().map(|count| count.summary())) {
            let _ = writeln!(stdout, "{count}");
        }
        return Ok(code);
    }
}

fn new_name(live: &[SessionInfo], tty: bool) -> Result<String, String> {
    let fallback = crate::proto::free_name(|name| live.iter().any(|s| s.label == name));
    if !tty {
        return Ok(fallback);
    }
    // On stdout, which is the terminal here (stderr may not be).
    client::install_signal_handlers();
    let term = console::Term::stdout();
    let name: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("session name")
        .default(fallback.clone())
        .interact_text_on(&term)
        .map_err(|_| {
            let _ = term.show_cursor();
            "cancelled".to_string()
        })?;
    let name = name.trim();
    Ok(if name.is_empty() { fallback } else { name.to_string() })
}

#[derive(Clone)]
enum Row {
    Session(SessionInfo),
    Tmux { host: String, tmux: TmuxInfo },
    Open { host: String },
    Down { host: String, error: String },
    New,
}

/// A row as menus show it. A host row heads the rows after it (`rsh ls`
/// groups by host), which are numbered from 1 under it; `empty` says no
/// session rows follow.
fn row_text(number: usize, row: &Row, empty: bool) -> String {
    match row {
        Row::Session(s) => {
            let state = if s.attached { "attached" } else { "detached" };
            let (label, title) = (printable(&s.label), printable(&s.title));
            let mut text = format!("[{number}] {} {label}  {state} idle {}", s.id, short_age(s.idle_secs));
            if !title.is_empty() {
                text.push_str(&format!("  {title}"));
            }
            text
        }
        Row::Tmux { tmux, .. } => format!("[{number}] tmux {}  {}", printable(&tmux.name), tmux_state(tmux, false)),
        Row::Open { host } if empty => format!("{}  no rsh sessions; open a new one", printable(host)),
        Row::Open { host } => printable(host),
        Row::Down { host, error } => format!("{}  unreachable: {}", printable(host), printable(error)),
        Row::New => format!("[{number}] new session"),
    }
}

/// Each row's text and its number under its host (0 for host rows).
fn numbered(rows: &[Row]) -> Vec<(usize, String)> {
    let mut number = 0;
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            if matches!(row, Row::Open { .. } | Row::Down { .. }) {
                number = 0;
                let empty = !matches!(rows.get(index + 1), Some(Row::Session(_)));
                (0, row_text(0, row, empty))
            } else {
                number += 1;
                (number, row_text(number, row, false))
            }
        })
        .collect()
}

/// Clients, size (with `size`) and idle time of a tmux session, and the
/// rsh session joined to it.
fn tmux_state(tmux: &TmuxInfo, size: bool) -> String {
    let size = if size { format!(" {}x{}", tmux.cols, tmux.rows) } else { String::new() };
    let mut text = format!(
        "{} client{}{size} idle {}",
        tmux.clients,
        if tmux.clients == 1 { "" } else { "s" },
        short_age(tmux.idle_secs)
    );
    if tmux.session != 0 {
        text.push_str(&format!("  rsh {}", tmux.session));
    }
    text
}

/// Text from the server with control characters removed, so a title a
/// program set cannot send escape sequences to this terminal.
pub(crate) fn printable(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

fn short_age(secs: u64) -> String {
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86400),
    }
}

/// Inline menu: draws below the cursor and redraws in place, so the
/// terminal's earlier contents stay. Returns None when there is no input.
fn choose_row(rows: &[Row], keys: &mut Keys, stdout: &mut dyn Write, tty: bool) -> Result<Option<usize>, String> {
    // The first session, else the first host that answers.
    let selected = rows
        .iter()
        .position(|row| matches!(row, Row::Session(_) | Row::Tmux { .. } | Row::New))
        .or_else(|| rows.iter().position(|row| !matches!(row, Row::Down { .. })))
        .unwrap_or(0);
    let shown = numbered(rows);
    let io_err = |err: io::Error| err.to_string();
    if !tty {
        for (_, text) in &shown {
            writeln!(stdout, "{text}").map_err(io_err)?;
        }
        write!(stdout, "select> ").map_err(io_err)?;
        stdout.flush().map_err(io_err)?;
        let mut line = Vec::new();
        loop {
            match keys.byte()? {
                None if line.is_empty() => {
                    writeln!(stdout).map_err(io_err)?;
                    return Ok(None);
                }
                None | Some(b'\n') => break,
                Some(byte) => line.push(byte),
            }
        }
        writeln!(stdout).map_err(io_err)?;
        let text = String::from_utf8_lossy(&line);
        let text = text.trim();
        if text == "q" {
            return Err("cancelled".to_string());
        }
        if text.is_empty() {
            return Ok(Some(selected));
        }
        return pick_typed(rows, &shown, text).map(Some).ok_or_else(|| format!("no row {text}"));
    }
    // Unreachable hosts are shown above the menu but cannot be picked.
    let mut choices = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if matches!(row, Row::Down { .. }) {
            writeln!(stdout, "  {}", shown[index].1).map_err(io_err)?;
        } else {
            choices.push(index);
        }
    }
    if choices.is_empty() {
        return Err("no host answers".to_string());
    }
    // One screen line each: a row that wraps confuses the menu's redraw.
    let width = usize::from(crate::pty::window_size(1).0).saturating_sub(6).max(20);
    // Rows under a host are indented below it.
    let labels: Vec<String> = choices
        .iter()
        .map(|&index| {
            let (number, text) = &shown[index];
            let text = if *number > 0 && rows.iter().any(|row| matches!(row, Row::Open { .. })) { format!("  {text}") } else { text.clone() };
            console::truncate_str(&text, width, "…").into_owned()
        })
        .collect();
    // Typing filters the rows; Esc or Ctrl-C cancels. Drawn on stdout,
    // the terminal here: on a stderr that is not one, it would spin.
    client::install_signal_handlers();
    let term = console::Term::stdout();
    let picked = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("rsh")
        .items(&labels)
        .default(choices.iter().position(|&index| index == selected).unwrap_or(0))
        .max_length(labels.len().clamp(3, 15))
        .interact_on_opt(&term);
    match picked {
        Ok(Some(index)) => Ok(Some(choices[index])),
        _ => {
            let _ = term.show_cursor();
            Err("cancelled".to_string())
        }
    }
}

/// The row a typed line names: `<n>` when one host answers, else
/// `<host> <n>`; `<host>` alone is its host row.
fn pick_typed(rows: &[Row], shown: &[(usize, String)], text: &str) -> Option<usize> {
    let (host, number) = match text.rsplit_once(' ') {
        Some((host, n)) => (Some(host.trim()), n.parse::<usize>().ok()?),
        None => match text.parse::<usize>() {
            Ok(n) => (None, n),
            Err(_) => return rows.iter().position(|row| matches!(row, Row::Open { host } if host == text)),
        },
    };
    let headers = rows.iter().filter(|row| matches!(row, Row::Open { .. })).count();
    let mut group = None;
    for (index, row) in rows.iter().enumerate() {
        match row {
            Row::Open { host: name } | Row::Down { host: name, .. } => group = Some(name.as_str()),
            _ if shown[index].0 == number && (host.is_none() && headers <= 1 || host.is_some() && host == group) => return Some(index),
            _ => {}
        }
    }
    None
}

fn reachable_rows(store: &AuthStore, opts: &Options) -> Vec<(Row, SocketAddr, Option<Key>)> {
    let hosts: Vec<String> = match &opts.host {
        Some(host) => vec![host.clone()],
        None => store.hosts(),
    };
    host_rows(store, hosts, &opts.ssh, opts.peer.as_deref())
}

fn host_rows(store: &AuthStore, hosts: Vec<String>, ssh: &str, peer: Option<&str>) -> Vec<(Row, SocketAddr, Option<Key>)> {
    let unknown = SocketAddr::from(([0, 0, 0, 0], 0));
    // Hosts are asked at once, so unreachable ones cost one timeout in total.
    let answers: Vec<_> = std::thread::scope(|scope| {
        let jobs: Vec<_> = hosts
            .into_iter()
            .filter_map(|host| store.token(&host).map(|token| (host, Key::from_token(token))))
            .map(|(host, key)| {
                let peer = peer_of(&host, store, ssh, peer);
                scope.spawn(move || {
                    let listed = peer
                        .as_ref()
                        .map_err(Clone::clone)
                        .and_then(|peer| net::list(*peer, &key, Duration::from_millis(1500), true).map_err(|err| {
                            if err.kind() == ErrorKind::TimedOut { "no reply".to_string() } else { err.to_string() }
                        }));
                    (host, peer, key, listed)
                })
            })
            .collect();
        jobs.into_iter().filter_map(|job| job.join().ok()).collect()
    });
    let mut rows = Vec::new();
    for (host, peer, key, listed) in answers {
        let peer = peer.unwrap_or(unknown);
        match listed {
            Ok(listing) => {
                // An rsh session joined to a tmux session shows on its tmux row.
                let live: Vec<SessionInfo> =
                    listing.sessions.into_iter().filter(|s| s.alive && s.host == host && !crate::tmux::is_wrapper(&s.command)).collect();
                // The host row heads them and opens a session there.
                rows.push((Row::Open { host: host.clone() }, peer, Some(key.clone())));
                for session in live {
                    rows.push((Row::Session(session), peer, Some(key.clone())));
                }
                for tmux in listing.tmux {
                    rows.push((Row::Tmux { host: host.clone(), tmux }, peer, Some(key.clone())));
                }
            }
            Err(error) => rows.push((Row::Down { host, error }, peer, None)),
        }
    }
    rows
}

/// What `~l` in a session shows: the sessions of every known host, by
/// host, with the current one marked. With --peer only that host is asked,
/// since the address says nothing about the others.
fn session_overview(store: &AuthStore, ssh: &str, peer: Option<&str>, host: &str, here: &Target) -> Vec<client::ListRow> {
    let hosts = match peer {
        Some(_) => vec![host.to_string()],
        None => store.hosts(),
    };
    let mut lines = Vec::new();
    let mut last = None;
    let text = |text: String| client::ListRow { text, target: None };
    let rows: Vec<Row> = host_rows(store, hosts, ssh, peer).into_iter().map(|(row, _, _)| row).collect();
    for (index, row) in rows.iter().cloned().enumerate() {
        let (name, line) = match row {
            Row::Session(s) => {
                let target = Target::Rsh(s.host.clone(), s.id);
                let mark = if target == *here { "*" } else { " " };
                let state = if s.attached { "attached" } else { "detached" };
                let mut line = format!(
                    " {mark} {} {:<12} {state} {}x{} idle {}",
                    s.id,
                    printable(&s.label),
                    s.cols,
                    s.rows,
                    short_age(s.idle_secs)
                );
                let title = printable(&s.title);
                if !title.is_empty() {
                    line.push_str(&format!("  {title}"));
                }
                (s.host, client::ListRow { text: line, target: Some(target) })
            }
            Row::Tmux { host: name, tmux } => {
                let target = Target::Tmux(name.clone(), tmux.name.clone());
                let joined = Target::Rsh(name.clone(), tmux.session);
                let mark = if target == *here || joined == *here { "*" } else { " " };
                let line = format!(" {mark} {:<6} {:<12} {}", "tmux", printable(&tmux.name), tmux_state(&tmux, true));
                (name, client::ListRow { text: line, target: Some(target) })
            }
            // The host line comes from the rows under it.
            Row::Open { host } if matches!(rows.get(index + 1), Some(Row::Session(_))) => {
                if last.as_ref() != Some(&host) {
                    lines.push(text(printable(&host)));
                    last = Some(host);
                }
                continue;
            }
            Row::Open { host } => (host, text("   no rsh sessions".to_string())),
            Row::Down { host, error } => (host, text(format!("   unreachable: {}", printable(&error)))),
            Row::New => continue,
        };
        if last.as_ref() != Some(&name) {
            lines.push(text(printable(&name)));
            last = Some(name);
        }
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(text("no known hosts".to_string()));
    }
    lines
}

fn list_and_attach(opts: &Options, keys: &mut Keys, stdout: &mut dyn Write, tty: bool) -> Result<i32, String> {
    let store = load_store(&opts.auth)?;
    let found = reachable_rows(&store, opts);
    if found.is_empty() {
        return Err("no known hosts; connect once with rsh <host>".to_string());
    }
    let rows: Vec<Row> = found.iter().map(|(row, _, _)| row.clone()).collect();
    let Some(index) = choose_row(&rows, keys, stdout, tty)? else {
        return Ok(0);
    };
    // Through the same way as `rsh <host>`, so the host is brought to
    // this build first when it runs another.
    match found[index].0.clone() {
        Row::Session(s) => {
            let mut picked = opts.clone();
            picked.attach_id = Some(s.id);
            connect_host(&s.host, &picked, keys, stdout, tty)
        }
        Row::Tmux { host, tmux } => {
            let mut picked = opts.clone();
            picked.tmux = true;
            picked.name = Some(tmux.name);
            connect_host(&host, &picked, keys, stdout, tty)
        }
        Row::Open { host } => connect_host(&host, opts, keys, stdout, tty),
        Row::Down { host, .. } => Err(format!("{host} unreachable")),
        Row::New => Err("cancelled".to_string()),
    }
}

fn kill(opts: &Options, stdout: &mut dyn Write) -> Result<i32, String> {
    let host = opts.host.clone().ok_or("usage: rsh kill <host> <id|name>")?;
    let target = opts.target.clone().ok_or("usage: rsh kill <host> <id|name>")?;
    let store = load_store(&opts.auth)?;
    let token = store.token(&host).ok_or("authentication required: no token for this host")?;
    let peer = peer_for(&host, &store, opts)?;
    let killed = kill_sessions(peer, &Key::from_token(token), &host, &target).map_err(|err| format!("{host}: {err}"))?;
    if killed.is_empty() {
        return Err(format!("{host}: no session {target}"));
    }
    for id in killed {
        writeln!(stdout, "killed {host} session {id}").map_err(|err| err.to_string())?;
    }
    Ok(0)
}

/// Asks the running `rsh serve` for this port to exec the binary at its
/// path again, keeping its sessions, and waits until the new one is up.
/// Returns the path the server ran again.
fn upgrade_server(opts: &Options, stdout: &mut dyn Write) -> Result<PathBuf, String> {
    let port = opts.bind.map_or(DEFAULT_PORT, |bind| bind.port());
    let pidfile = crate::upgrade::pidfile(&opts.authorized, port);
    let server = crate::upgrade::running(&pidfile)
        .ok_or_else(|| format!("no rsh serve running for port {port} ({})", pidfile.display()))?;
    unsafe {
        libc::kill(server.pid, libc::SIGUSR2);
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(100));
        if unsafe { libc::kill(server.pid, 0) } != 0 {
            return Err(format!("rsh serve (pid {}) exited instead of upgrading", server.pid));
        }
        if let Some(now) = crate::upgrade::running(&pidfile) {
            if now.pid == server.pid && now.text != server.text {
                writeln!(
                    stdout,
                    "rsh serve upgraded in place (pid {}, {}); sessions kept",
                    now.pid,
                    now.exe.display()
                )
                .map_err(|err| err.to_string())?;
                return Ok(now.exe);
            }
        }
    }
    Err(format!(
        "rsh serve (pid {}) did not restart and keeps running the old binary; see {}",
        server.pid,
        home().join(".rsh").join("serve.log").display()
    ))
}

/// Copies this binary over `dest` by rename, never by rewriting the file a
/// running server may be executing.
fn install_binary(exe: &Path, dest: &Path) -> Result<(), String> {
    if exe == dest {
        return Ok(());
    }
    let tmp = dest.with_extension("new");
    fs::copy(exe, &tmp).map_err(|err| format!("{}: {err}", tmp.display()))?;
    fs::rename(&tmp, dest).map_err(|err| format!("{}: {err}", dest.display()))
}

fn install_launch_agent(opts: &Options, stdout: &mut dyn Write) -> Result<(), String> {
    let home = env::var_os("HOME").map(PathBuf::from).ok_or("HOME required")?;
    let exe = env::current_exe().map_err(|err| err.to_string())?;
    let dest_dir = home.join(".rsh").join("bin");
    net::private_dir(&dest_dir).map_err(|err| err.to_string())?;
    let dest = dest_dir.join("rsh");
    install_binary(&exe, &dest)?;
    let bind = opts.bind.unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT)));
    let plist = write_launch_agent(&home, &dest, &bind.to_string(), &opts.authorized, opts.disconnect_timeout)
        .map_err(|err| err.to_string())?;
    let pidfile = crate::upgrade::pidfile(&opts.authorized, bind.port());
    if crate::upgrade::running(&pidfile).is_some() {
        // Restarting through launchd would end every session; upgrade instead.
        let ran = upgrade_server(opts, stdout)?;
        let same = fs::canonicalize(&ran).ok() == fs::canonicalize(&dest).ok();
        if !same {
            writeln!(
                stdout,
                "note: that server was started from {}, so it ran that file again, not {}",
                ran.display(),
                dest.display()
            )
            .map_err(|err| err.to_string())?;
        }
        writeln!(stdout, "rsh serve installed {}", plist.display()).map_err(|err| err.to_string())?;
        return Ok(());
    }
    bootstrap_launch_agent(&plist)?;
    writeln!(stdout, "rsh serve installed {}", plist.display()).map_err(|err| err.to_string())?;
    writeln!(stdout, "rsh serving {bind}").map_err(|err| err.to_string())?;
    Ok(())
}

pub(crate) fn write_launch_agent(
    home: &Path,
    bin: &Path,
    bind: &str,
    authorized: &Path,
    disconnect_timeout: Option<Duration>,
) -> io::Result<PathBuf> {
    let timeout = disconnect_timeout
        .map(|limit| format!("\n    <string>--disconnect-timeout</string>\n    <string>{}</string>", limit.as_secs()))
        .unwrap_or_default();
    let dir = home.join("Library").join("LaunchAgents");
    fs::create_dir_all(&dir)?;
    let path = dir.join("com.rsh.serve.plist");
    let log = home.join(".rsh").join("serve.log");
    crate::net::private_dir(log.parent().unwrap_or(home))?;
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.rsh.serve</string>
  <key>ProgramArguments</key>
  <array>
    <string>{bin}</string>
    <string>serve</string>
    <string>--bind</string>
    <string>{bind}</string>
    <string>--authorized</string>
    <string>{auth}</string>{timeout}
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{log}</string>
  <key>EnvironmentVariables</key>
  <dict>
    <key>HOME</key>
    <string>{home}</string>
    <key>LANG</key>
    <string>en_US.UTF-8</string>
  </dict>
</dict>
</plist>
"#,
        bin = xml_escape(&bin.display().to_string()),
        bind = xml_escape(bind),
        auth = xml_escape(&authorized.display().to_string()),
        log = xml_escape(&log.display().to_string()),
        home = xml_escape(&home.display().to_string()),
    );
    fs::write(&path, body)?;
    Ok(path)
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn bootstrap_launch_agent(plist: &Path) -> Result<(), String> {
    let uid = unsafe { libc::getuid() };
    let domain = format!("gui/{uid}");
    let label = format!("{domain}/com.rsh.serve");
    let _ = Command::new("launchctl").args(["bootout", &label]).status();
    let boot = Command::new("launchctl")
        .args(["bootstrap", &domain])
        .arg(plist)
        .status()
        .map_err(|err| err.to_string())?;
    if boot.success() {
        return Ok(());
    }
    let load = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(plist)
        .status()
        .map_err(|err| err.to_string())?;
    if load.success() {
        Ok(())
    } else {
        Err(format!("launchctl bootstrap exited {boot}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn setup(name: &str) -> (Server, PathBuf, PathBuf) {
        let server = Server::spawn();
        server.allow("fixture.example", "secret-token");
        let dir = std::env::temp_dir().join(format!("rsh-cli-{name}-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let auth = dir.join("auth");
        let mut store = AuthStore::new();
        store.insert("fixture.example", "secret-token");
        store.save(&auth).unwrap();
        (server, dir, auth)
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn marker(text: &str) -> u64 {
        let start = text.rfind("MARK:").unwrap_or_else(|| panic!("no marker in {text}")) + 5;
        text[start..].chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap()
    }

    fn run_text(list: &[&str], stdin: &str) -> (i32, String) {
        let mut out = Vec::new();
        let code = run(&args(list), &mut Cursor::new(stdin.as_bytes().to_vec()), &mut out).unwrap();
        (code, String::from_utf8_lossy(&out).into_owned())
    }

    #[test]
    fn cli_host_banner_has_no_auth_prompt() {
        let (server, dir, auth) = setup("host");
        let addr = server.addr().to_string();
        let auth = auth.display().to_string();
        let host = ["--auth", &auth, "--peer", &addr, "fixture.example"];
        let (code, text) = run_text(&host, "echo MARK:$RSH_SESSION\n");
        assert_eq!(code, 0);
        assert!(!text.to_lowercase().contains("password"), "{text}");
        assert!(!text.contains("authentication required"));
        let (_, again) = run_text(&host, "echo MARK:$RSH_SESSION\n");
        assert_eq!(marker(&again), marker(&text));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ls_rows_are_grouped_and_numbered_by_host() {
        let session = |id: u64, host: &str| {
            Row::Session(SessionInfo {
                id,
                host: host.into(),
                label: format!("s{id}"),
                alive: true,
                attached: false,
                cols: 80,
                rows: 24,
                created_unix: 0,
                idle_secs: 5,
                command: String::new(),
                title: String::new(),
            })
        };
        let rows = vec![
            Row::Open { host: "a".into() },
            session(111111, "a"),
            session(222222, "a"),
            Row::Open { host: "b".into() },
            Row::Open { host: "t".into() },
            Row::Tmux { host: "t".into(), tmux: TmuxInfo { name: "main".into(), clients: 1, cols: 80, rows: 24, idle_secs: 2, session: 0 } },
            Row::Down { host: "c".into(), error: "no reply".into() },
        ];
        let shown: Vec<String> = numbered(&rows).into_iter().map(|(_, text)| text).collect();
        assert_eq!(
            shown,
            [
                "a",
                "[1] 111111 s111111  detached idle 5s",
                "[2] 222222 s222222  detached idle 5s",
                "b  no rsh sessions; open a new one",
                "t  no rsh sessions; open a new one",
                "[1] tmux main  1 client idle 2s",
                "c  unreachable: no reply",
            ]
        );
        let numbered = numbered(&rows);
        assert_eq!(pick_typed(&rows, &numbered, "a 2"), Some(2));
        assert_eq!(pick_typed(&rows, &numbered, "b"), Some(3));
        assert_eq!(pick_typed(&rows, &numbered, "t 1"), Some(5));
        assert_eq!(pick_typed(&rows, &numbered, "2"), None, "several hosts: the number needs its host");
        assert_eq!(pick_typed(&rows[..3], &numbered[..3], "2"), Some(2));
        // An unreachable host does not make the number ambiguous.
        let one = [rows[0].clone(), rows[1].clone(), rows[6].clone()];
        assert_eq!(pick_typed(&one, &super::numbered(&one), "1"), Some(1));
    }

    #[test]
    fn cli_ls_selects_second_session() {
        let (server, dir, auth) = setup("ls");
        let addr = server.addr().to_string();
        let auth = auth.display().to_string();
        run_text(&["--auth", &auth, "--peer", &addr, "--name", "one", "fixture.example"], "true\n");
        run_text(&["--auth", &auth, "--peer", &addr, "--name", "two", "fixture.example"], "true\n");
        let ls = ["ls", "--auth", &auth, "--peer", &addr];
        let (_, listing) = run_text(&ls, "");
        // Grouped under the host; tmux rows (the user's own tmux, if any) come after.
        assert!(listing.lines().any(|line| line == "fixture.example"), "{listing}");
        let rows: Vec<&str> = listing.lines().filter(|line| line.starts_with('[') && !line.contains("] tmux ")).collect();
        assert_eq!(rows.len(), 2, "{listing}");
        assert!(rows[0].starts_with("[1] ") && rows[1].starts_with("[2] "), "{listing}");
        let second: u64 = rows[1].split_whitespace().nth(1).unwrap().parse().unwrap();
        let (_, picked) = run_text(&ls, "2\necho MARK:$RSH_SESSION\n");
        assert_eq!(marker(&picked), second, "{picked}");
        let (_, again) = run_text(&ls, "2\necho MARK:$RSH_SESSION\n");
        assert_eq!(marker(&again), second);
        let (code, killed) = run_text(&["kill", "fixture.example", "two", "--auth", &auth, "--peer", &addr], "");
        assert_eq!(code, 0);
        assert!(killed.contains(&format!("killed fixture.example session {second}")), "{killed}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn shell_survives_loss_and_server_rebind() {
        let (server, dir, auth) = setup("recover");
        let auth = auth.display().to_string();
        let with = |addr: &str| -> Vec<String> {
            args(&["--auth", &auth, "--peer", addr, "--name", "work", "fixture.example"])
        };
        let mut out = Vec::new();
        run(&with(&server.addr().to_string()), &mut Cursor::new("export RSH_MARK=kept\n"), &mut out).unwrap();
        server.set_drop_incoming(true);
        let mut lost = Vec::new();
        assert!(run(&with(&server.addr().to_string()), &mut Cursor::new("echo MARK:$RSH_MARK\n"), &mut lost).is_err());
        server.set_drop_incoming(false);
        let mut restored = Vec::new();
        run(&with(&server.addr().to_string()), &mut Cursor::new("echo MARK:$RSH_MARK\n"), &mut restored).unwrap();
        assert!(String::from_utf8_lossy(&restored).contains("MARK:kept"));
        let moved = server.rebind().unwrap();
        let mut again = Vec::new();
        run(&with(&moved.to_string()), &mut Cursor::new("echo MARK:$RSH_MARK\n"), &mut again).unwrap();
        assert!(String::from_utf8_lossy(&again).contains("MARK:kept"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn command_exit_status_is_returned() {
        let (server, dir, auth) = setup("cmd");
        let addr = server.addr().to_string();
        let auth = auth.display().to_string();
        let (code, text) = run_text(&["--auth", &auth, "--peer", &addr, "fixture.example", "--", "echo", "ran;", "exit", "5"], "");
        assert_eq!(code, 5);
        assert!(text.contains("ran"), "{text}");
        let named = ["--auth", &auth, "--peer", &addr, "--name", "job", "fixture.example", "--", "sleep", "5"];
        let mut out = Vec::new();
        let _ = run(&args(&named), &mut Cursor::new(Vec::new()), &mut out);
        let (_, listing) = run_text(&["ls", "--auth", &auth, "--peer", &addr], "");
        assert!(listing.contains(" job "), "{listing}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn help_and_parse() {
        let (_, text) = run_text(&["--help"], "");
        for command in ["<HOST>", "ls", "kill", "serve", "bootstrap", "--name", "--auth", "--peer", "--bind", "~.", "kept until they exit"] {
            assert!(text.contains(command), "{command} in {text}");
        }
        let opts = Options::parse(&args(&["host", "--name", "x", "--", "ls", "-l"])).unwrap();
        assert_eq!(opts.host.as_deref(), Some("host"));
        assert_eq!(opts.run, args(&["ls", "-l"]));
        assert_eq!(Options::parse(&args(&["host", "work"])).unwrap().name.as_deref(), Some("work"));
        assert!(Options::parse(&args(&["host", "work", "extra"])).is_err());
        assert!(Options::parse(&args(&["host", "work", "--name", "other"])).is_err());
        let named = Options::parse(&args(&["host", "serve"])).unwrap();
        assert!(named.command == Sub::Connect && named.name.as_deref() == Some("serve"), "a subcommand word after the host is a name");
        assert_eq!(Options::parse(&args(&["host", "--name=serve"])).unwrap().name.as_deref(), Some("serve"));
        assert_eq!(resume_command("h", "work", false), "rsh h work");
        assert_eq!(resume_command("h", "serve", false), "rsh h --name=serve");
        assert_eq!(resume_command("h", "-x", false), "rsh h --name=-x");
        assert_eq!(resume_command("h", "main", true), "rsh h main --tmux");
        assert_eq!(Options::parse(&args(&["h", "--name=-x"])).unwrap().name.as_deref(), Some("-x"));
        assert!(Options::parse(&args(&["--bogus"])).is_err());
        let serve = Options::parse(&args(&["serve", "upgrade", "--bind", "127.0.0.1:1", "--authorized", "/x"])).unwrap();
        assert!(serve.command == Sub::Upgrade && serve.bind.is_some() && !serve.default_files);
        assert!(Options::parse(&args(&["--bind", "127.0.0.1:1", "bootstrap", "--client", "me@box"])).unwrap().command == Sub::Bootstrap);
        let kill = Options::parse(&args(&["kill", "box", "two"])).unwrap();
        assert_eq!((kill.host.as_deref(), kill.target.as_deref()), (Some("box"), Some("two")));
        assert!(Options::parse(&args(&["--version"])).unwrap().command == Sub::Version);
        let (code, text) = run_text(&["serve", "--help"], "");
        assert!(code == 0 && text.contains("upgrade"), "{text}");
        assert_eq!(crate::remote::shell_quote("a b'c"), "'a b'\\''c'");
        assert_eq!(ssh_host("me@box"), "box");
        assert_eq!(printable("a\x1b]2;x\x07b\u{9b}c"), "a]2;xbc");
    }

    #[test]
    fn bootstrap_prints_token_and_starts_server() {
        let dir = std::env::temp_dir().join(format!("rsh-boot-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let auth = dir.join("auth");
        let sock = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = sock.local_addr().unwrap().port();
        drop(sock);
        let bind = format!("127.0.0.1:{port}");
        let auth_text = auth.display().to_string();
        let server = Server::start(bind.parse().unwrap(), &AuthStore::new(), Some(auth.clone()), None).unwrap();
        let (code, text) = run_text(&["bootstrap", "--authorized", &auth_text, "--bind", &bind, "--client", "me@box"], "");
        assert_eq!(code, 0);
        let line = text.lines().find(|l| l.starts_with("RSH CONNECT ")).unwrap();
        let token = line.split_whitespace().nth(3).unwrap().to_string();
        assert_eq!(line.split_whitespace().nth(2).unwrap(), port.to_string());
        assert_eq!(AuthStore::load(&auth).unwrap().token("client:me@box"), Some(token.as_str()));
        let (_, again) = run_text(&["bootstrap", "--authorized", &auth_text, "--bind", &bind, "--client", "me@box"], "");
        assert!(again.contains(&token));
        drop(server);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bootstrap_links_rsh_into_local_bin() {
        use std::os::unix::fs::PermissionsExt;
        let home = std::env::temp_dir().join(format!("rsh-link-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let exe = home.join(".rsh").join("bin").join("rsh");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, "").unwrap();
        // A shell whose PATH has ~/.local/bin, and one whose PATH does not.
        let shell = |name: &str, path: &str| {
            let file = home.join(name);
            fs::write(&file, format!("#!/bin/sh\necho banner\nprintf '\\nRSH-PATH:%s\\n' '{path}'\n")).unwrap();
            fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
            file.display().to_string()
        };
        let good = shell("good", &format!("/usr/bin:{}", home.join(".local/bin").display()));
        let bad = shell("bad", "/usr/bin:/bin");
        // An rc file that leaves a job running does not hold bootstrap, and
        // the shell sees a clean environment, not this process's PATH.
        let slow = home.join("slow");
        fs::write(&slow, "#!/bin/sh\nsleep 30 &\nprintf '\\nRSH-PATH:%s\\n' \"$PATH\"\nsleep 30\n").unwrap();
        fs::set_permissions(&slow, fs::Permissions::from_mode(0o755)).unwrap();
        let started = Instant::now();
        let note = link_on_path(&home, &exe, &slow.display().to_string());
        assert!(started.elapsed() < Duration::from_secs(2), "{:?}", started.elapsed());
        assert!(note.unwrap().contains("not on PATH"), "the answer counts though the job runs on");
        let mute = home.join("mute");
        fs::write(&mute, "#!/bin/sh\nsleep 30\n").unwrap();
        fs::set_permissions(&mute, fs::Permissions::from_mode(0o755)).unwrap();
        let started = Instant::now();
        assert_eq!(link_on_path(&home, &exe, &mute.display().to_string()), None, "no answer in time: no note");
        assert!(started.elapsed() < Duration::from_secs(5));
        let env = home.join("env");
        fs::write(&env, "#!/bin/sh\nprintf '\\nRSH-PATH:%s\\n' \"$PATH\"\n").unwrap();
        fs::set_permissions(&env, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(link_on_path(&home, &exe, &env.display().to_string()).unwrap().contains("not on PATH"));
        assert_eq!(link_on_path(&home, &exe, &good), None);
        let link = home.join(".local/bin/rsh");
        assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(&exe).unwrap());
        assert!(link_on_path(&home, &exe, &bad).unwrap().contains("not on PATH"));
        // A link to a removed rsh is replaced; another program is left alone.
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(home.join("gone"), &link).unwrap();
        assert_eq!(link_on_path(&home, &exe, &good), None);
        assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(&exe).unwrap());
        fs::remove_file(&link).unwrap();
        fs::write(&link, "mine").unwrap();
        assert!(link_on_path(&home, &exe, &good).unwrap().contains("another program"));
        assert_eq!(fs::read_to_string(&link).unwrap(), "mine");
        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn serve_launch_agent_runs_at_load() {
        let home = std::env::temp_dir().join(format!("rsh-agent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let bin = home.join("bin").join("rsh");
        let authorized = home.join("auth").join("authorized");
        let path = write_launch_agent(&home, &bin, "0.0.0.0:2222", &authorized, Some(Duration::from_secs(9))).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("<key>RunAtLoad</key>"));
        assert!(text.contains("<key>KeepAlive</key>"));
        assert!(text.contains("<string>serve</string>"));
        assert!(text.contains("<string>0.0.0.0:2222</string>"));
        assert!(text.contains(&bin.display().to_string()));
        assert!(text.contains(&format!("<string>{}</string>", authorized.display())));
        assert!(text.contains("<string>--disconnect-timeout</string>\n    <string>9</string>"));
        let _ = fs::remove_dir_all(&home);
    }
}
