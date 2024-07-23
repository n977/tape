use anyhow::{bail, Context, Result};
use std::fs::{DirEntry, File};
use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use tape::engine::PlaybackState;
use tape::factory::{FactoryState, TranslateBehavior};
use tape::Request;
use tape::{Engine, Factory, Sound};
use tracing::{debug, error, warn};

fn main() {
    if let Err(e) = run() {
        error!("{:#}", e);
        std::process::exit(1);
    }
}

pub fn run() -> Result<()> {
    tape::logger::init()?;

    debug!("Audio player daemon {}", env!("CARGO_PKG_VERSION"));

    let dir = tape::runtime_dir()?;
    let path = tape::socket_path()?;

    std::fs::create_dir_all(dir)
        .with_context(|| format!("{}: failed to create runtime directory", path.display()))?;

    if let Err(e) = std::fs::remove_file(&path) {
        match e.kind() {
            std::io::ErrorKind::NotFound => (),
            _ => {
                let e = Err(e)
                    .with_context(|| format!("{}: failed to clear stale socket", path.display()));

                return e;
            }
        }
    }

    let socket = UnixListener::bind(&path)
        .with_context(|| format!("{}: failed to bind to socket", path.display()))?;

    let mut server = Server::new()?;
    server.run(&socket);

    Ok(())
}

struct Server {
    engine: Engine<Factory<Sound>>,
    buf: Vec<u8>,
}

impl Server {
    fn new() -> Result<Self> {
        let factory = Factory::new();
        let mut engine = Engine::new(factory)?;
        engine.run()?;

        let server = Self {
            engine,
            buf: Vec::new(),
        };

        Ok(server)
    }

    fn serve(&mut self, con: &mut UnixStream) -> Result<()> {
        self.buf.clear();
        con.read_to_end(&mut self.buf)?;

        let req =
            serde_json::from_slice::<Request>(&self.buf).context("failed to accept request")?;

        match req {
            Request::Add { paths } => {
                let mut files = Vec::new();

                for path in paths {
                    if path.is_file() {
                        files.push(path.clone());
                    } else {
                        for entry in std::fs::read_dir(path)? {
                            let path = match process_entry(entry).context("failed to read path") {
                                Ok(path) => path,
                                Err(e) => {
                                    warn!("{:#}", e);
                                    continue;
                                }
                            };
                            files.push(path);
                        }
                    }
                }

                for path in files {
                    match probe_file(&path)
                        .with_context(|| format!("{}: failed to probe file", path.display()))
                    {
                        Ok(sound) => self.engine.provider().map(|items| items.push(sound)),
                        Err(e) => warn!("{:#}", e),
                    }
                }

                self.engine.state().set(PlaybackState::Playing);
            }
            Request::Remove { ids } => self.engine.provider().map(|items| {
                let mut d = 0;

                for i in &ids {
                    let i = match i.checked_sub(d) {
                        Some(i) if i < items.len() => i,
                        _ => continue,
                    };

                    items.remove(i);
                    d += 1;
                }
            }),
            Request::Config { props } => {
                let mut state = self.engine.provider().state();
                let mut ser = serde_json::to_value(&*state)?;

                for (key, value) in props {
                    if let Some(prop) = ser.get_mut(key) {
                        *prop = value.into();
                    }
                }

                let de = serde_json::from_value::<FactoryState>(ser)
                    .context("failed to update state")?;
                state.replace(de);
            }
            Request::Seek { t } => self.engine.provider().seek(t),
            Request::Jump { pos, relative } => {
                if relative {
                    self.engine
                        .provider()
                        .translate(pos, TranslateBehavior::Free);
                } else if let Ok(pos) = pos.try_into() {
                    self.engine.provider().select(pos);
                }
            }
            Request::Play => self.engine.state().set(PlaybackState::Playing),
            Request::Pause => self.engine.state().set(PlaybackState::Paused),
        }
        Ok(())
    }

    fn run(&mut self, sock: &UnixListener) {
        for con in sock.incoming() {
            if let Err(e) = con
                .context("failed to accept incoming connection")
                .and_then(|mut con| self.serve(&mut con))
            {
                warn!("{:#}", e);
            }
        }
    }
}

fn probe_file<P>(path: P) -> Result<Sound>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path)?;
    let sound = Sound::new(file)?;

    Ok(sound)
}

fn process_entry(entry: std::io::Result<DirEntry>) -> Result<PathBuf> {
    let entry = entry?;
    let path = entry.path();

    if path.is_dir() {
        bail!("not a file");
    }

    Ok(path)
}
