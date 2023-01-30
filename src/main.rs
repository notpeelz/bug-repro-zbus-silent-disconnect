use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_interface, Connection};

fn main() {
  fern::Dispatch::new()
    .format(|out, message, record| {
      out.finish(format_args!("[foobar {}]: {}", record.level(), message))
    })
    .level(log::LevelFilter::Debug)
    .chain(std::io::stderr())
    .apply()
    .unwrap();

  match tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .build()
  {
    Ok(rt) => {
      rt.block_on(async {
        if let Err(err) = crate::listen_dbus().await {
          log::error!("something bad happened: {err}");
        } else {
          log::info!("done?");
        }
      });
    },
    Err(err) => {
      log::error!("failed to create tokio runtime: {err}");
    },
  }
}

#[derive(
  Default,
  Clone,
  Copy,
  // XXX: this causes zbus to silently disconnect
  Deserialize,
  Serialize,
  // Deserialize_repr,
  // Serialize_repr,
  zbus::zvariant::Type,
  zbus::zvariant::Value,
)]
#[repr(u8)]
pub(crate) enum Blah {
  #[default]
  A = 0b00,
  B = 0b01,
  C = 0b10,
}

impl TryFrom<u8> for Blah {
  type Error = ();

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      x if x == Blah::A as u8 => Ok(Blah::A),
      x if x == Blah::B as u8 => Ok(Blah::B),
      x if x == Blah::C as u8 => Ok(Blah::C),
      _ => Err(()),
    }
  }
}

#[derive(Default)]
pub(crate) struct MyInterface {}

#[dbus_interface(name = "dev.peelz.FooBar.Baz")]
impl MyInterface {
  #[dbus_interface(out_args("a", "b"))]
  fn do_thing(&mut self) -> zbus::fdo::Result<(Blah, Blah)> {
    log::info!("doing the thing");
    Ok((Blah::B, Blah::C))
  }
}

async fn listen_dbus() -> Result<()> {
  const DBUS_SESSION_BUS_ADDRESS: &str = "DBUS_SESSION_BUS_ADDRESS";
  if std::env::var(DBUS_SESSION_BUS_ADDRESS).is_err() {
    log::warn!("env var '{DBUS_SESSION_BUS_ADDRESS}' is not set or is invalid");
  }

  let connection = Connection::session().await?;
  let object_server = connection.object_server();

  const PATH: &str = "/dev/peelz/FooBar";
  object_server
    .at(PATH, MyInterface::default())
    .await?;

  connection
    .request_name("dev.peelz.foobar")
    .await?;

  std::future::pending::<()>().await;
  unreachable!();
}
