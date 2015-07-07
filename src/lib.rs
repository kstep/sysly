#![deny(missing_docs)]
//#![cfg_attr(test, deny(warnings))]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

//! sysly is a rust interface for [syslog](https://tools.ietf.org/html/rfc5424)

//  #[cfg(all(test, feature = "nightly"))]
//  extern crate test;
extern crate time;
extern crate unix_socket;

use std::convert::AsRef;
use std::io::{ self, Write };
use std::net::{ Ipv4Addr, UdpSocket, SocketAddr, SocketAddrV4 };
use std::path::Path;
use std::ops::Deref;
use time::Tm;
use unix_socket::UnixStream;

static NIL: &'static str = "-";

/// Syslog [Facilities](https://tools.ietf.org/html/rfc5424#page-10)
#[derive(Copy,Clone)]
pub enum Facility {
  /// kernal facility
  KERN     = 0,
  /// user facility
  USER     = 1 << 3,
  /// user facility
  MAIL     = 2 << 3,
  /// daemon facility
  DAEMON   = 3 << 3,
  /// auth facility
  AUTH     = 4 << 3,
  /// syslog facility
  SYSLOG   = 5 << 3,
  /// lineptr facility
  LINEPTR  = 6 << 3,
  /// news facility
  NEWS     = 7 << 3,
  /// uucp facility
  UUCP     = 8 << 3,
  /// clock facility
  CLOCK    = 9 << 3,
  /// auth facility
  AUTHPRIV = 10 << 3,
  /// ftp facility
  FTP      = 11 << 3,
  /// Local0 facility
  LOCAL0   = 16 << 3,
  /// Local1 facility
  LOCAL1   = 17 << 3,
  /// Local2 facility
  LOCAL2   = 18 << 3,
  /// Local3 facility
  LOCAL3   = 19 << 3,
  /// Local4 facility
  LOCAL4   = 20 << 3,
  /// Local5 facility
  LOCAL5   = 21 << 3,
  /// Local6 facility
  LOCAL6   = 22 << 3,
  /// Local7 facility
  LOCAL7   = 23 << 3
}

/// Syslog [Severities](https://tools.ietf.org/html/rfc5424#page-11)
pub enum Severity {
  /// Emergency Severity
  EMERGENCY,
  /// Alert Severity
  ALERT,
  /// Critical Severity
  CRITICAL,
  /// Error Severity
  ERROR,
  /// Warning Severity
  WARNING,
  /// Notice Severity
  NOTICE,
  /// Info Severity
  INFO,
  /// Debug Severity
  DEBUG
}

/// Result of log operations
pub type Result = io::Result<()>;

trait Transport {
  fn send(&mut self, line: &str) -> Result;
}

impl Transport for (UdpSocket, SocketAddr) {
  fn send(&mut self, line: &str) -> Result {
    self.0.send_to(line.as_bytes(), &self.1).map(|_| ())
  }
}

impl Transport for UnixStream {
  fn send(&mut self, line: &str) -> Result {
    self.write_all(line.as_bytes())
  }
}

/// A rust interface for Syslog, a standard unix system logging service
pub struct Syslog {
  /// A Syslog facility to target when logging
  facility: Facility,
  /// A Syslog host entry as defined by
  /// [rfc5424#section-6.2.4](https://tools.ietf.org/html/rfc5424#section-6.2.4)
  host: Option<String>,
  /// An optional app-name appended to Syslog messages as defined by 
  /// [rfc5424#section-6.2.5](https://tools.ietf.org/html/rfc5424#section-6.2.5)
  app: Option<String>,
  /// An optional proc-id appended to Syslog messages as defined by
  /// [rfc5424#section-6.2.6](https://tools.ietf.org/html/rfc5424#section-6.2.6)
  pid: Option<String>,
  /// An optional msg-id appended to Syslog messages as defined by 
  /// [rfc5424#section-6.2.7](https://tools.ietf.org/html/rfc5424#section-6.2.7)
  msgid: Option<String>,
  transport: Box<Transport>
}

impl Syslog {
   /// Factory for a Syslog appender that writes to
   /// remote Syslog daemon listening a SocketAddr
   pub fn udp(host: SocketAddr) -> Syslog {
     let socket =
       match UdpSocket::bind("0.0.0.0:0") {
         Err(e) => panic!("error binding to local addr {}", e),
         Ok(s) => s
       };
      let tup = (socket, host);
      Syslog {
        facility: Facility::USER,
        host: None,
        app: None,
        pid: None,
        msgid: None,
        transport: Box::new(tup)
      }
  }

  /// Same as udp with providing local loopback address with the standard syslog port
  pub fn localudp() -> Syslog {
    Syslog::udp(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 514)))
  }

  /// Factory for a Syslog appender that writes
  /// to a host-local Syslog daemon listening on a unix socket domain
  /// hosted at the given Path
  pub fn unix<P: AsRef<Path>>(path: P) -> Syslog {
    let stream =
      match UnixStream::connect(path) {
        Err(_) => panic!("failed to connect to socket"),
        Ok(s)  => s
      };
    Syslog {
      facility: Facility::USER,
      host: None,
      app: None,
      pid: None,
      msgid: None,
      transport: Box::new(stream)
    }
  }
  /// Returns a new Syslog appender configured to append with
  /// the provided Facility
  pub fn facility(self, facility: Facility) -> Syslog {
    Syslog {
      facility: facility,
      host: self.host,
      app: self.app,
      pid: self.pid,
      msgid: self.msgid,
      transport: self.transport
    }
  }

  /// Returns a new Syslog appender configured to append with
  /// the provided host addr
  pub fn host(self, local: &str) -> Syslog {
    Syslog {
      facility: self.facility,
      host: Some(local.to_owned()),
      app: self.app,
      pid: self.pid,
      msgid: self.msgid,
      transport: self.transport
    }
  }

  /// Returns a new Syslog appender, configured to append with
  /// the provided app-name
  pub fn app(self, app: &str) -> Syslog {
    Syslog {
      facility: self.facility,
      host: self.host,
      app: Some(app.to_owned()),
      pid: self.pid,
      msgid: self.msgid,
      transport: self.transport
    }
  }

  /// Returns a new Syslog appender configured to append with
  /// the provided p(rocess)id
  pub fn pid(self, pid: &str) -> Syslog {
    Syslog {
      facility: self.facility,
      host: self.host,
      app: self.app,
      pid: Some(pid.to_owned()),
      msgid: self.msgid,
      transport: self.transport
    }
  }

  /// Returns a new Syslog appender configured to append with
  /// the provided msgid
  pub fn msgid(self, id: &str) -> Syslog {
    Syslog {
      facility: self.facility,
      host: self.host,
      app: self.app,
      pid: self.pid,
      msgid: Some(id.to_string()),
      transport: self.transport
    }
  }

  /// Emits a debug level message
  pub fn debug(&mut self, msg: &str) -> Result {
    self.log(Severity::DEBUG, msg)
  }

  /// Emits an info level message
  pub fn info(&mut self, msg: &str) -> Result {
    self.log(Severity::INFO, msg)
  }

  /// Emits an info level message
  pub fn notice(&mut self, msg: &str) -> Result {
    self.log(Severity::NOTICE, msg)
  }

  /// Emits an warn level message
  pub fn warn(&mut self, msg: &str) -> Result {
    self.log(Severity::WARNING, msg)
  }

  /// Emits an error level message
  pub fn err(&mut self, msg: &str) -> Result {
    self.log(Severity::ERROR, msg)
  }

  /// Emits a critical level message
  pub fn critical(&mut self, msg: &str) -> Result {
    self.log(Severity::CRITICAL, msg)
  }

  /// Emits an alert level message
  pub fn alert(&mut self, msg: &str) -> Result {
    self.log(Severity::ALERT, msg)
  }

  /// Emits a emergencycritical level message
  pub fn emergency(&mut self, msg: &str) -> Result {
    self.log(Severity::EMERGENCY, msg)
  }

  fn log(&mut self, severity: Severity,  msg: &str) -> Result {
    let formatted = Syslog::line(
        self.facility.clone(), severity, time::now(), self.host.as_ref().map(Deref::deref), self.app.as_ref().map(Deref::deref), self.pid.as_ref().map(Deref::deref), self.msgid.as_ref().map(Deref::deref), msg);
    self.transport.send(&formatted)
  }

  fn line(facility: Facility, severity: Severity, timestamp: Tm, host: Option<&str>, app: Option<&str>, pid: Option<&str>, msgid: Option<&str>, msg: &str) -> String {
    format!(
      "<{:?}>1 {} {} {} {} {} {}",
        Syslog::priority(facility, severity),
        timestamp.rfc3339(),
        host.unwrap_or(NIL),
        app.unwrap_or(NIL),
        pid.unwrap_or(NIL),
        msgid.unwrap_or(NIL),
        msg)
  }

  // computes the priority of a message based on a facility and severity
  fn priority(facility: Facility, severity: Severity) -> u8 {
    facility as u8 | severity as u8
  }
}

#[cfg(test)]
mod tests {
  use super::{Syslog, Facility, Severity};
  use time;
  //use test::Bencher;

  #[test]
  fn test_syslog_line_defaults() {
    let ts = time::now();
    assert_eq!(Syslog::line(
      Facility::LOCAL0, Severity::INFO, ts, None, None, None, None, "yo"),
      format!("<134>1 {} - - - - yo", ts.rfc3339()));
  }

  #[test]
  fn test_syslog_line_host() {
    let ts = time::now();
    let host = "foo.local";
    assert_eq!(Syslog::line(
      Facility::LOCAL0, Severity::INFO, ts, Some(host), None, None, None, "yo"),
      format!("<134>1 {} {} - - - yo", ts.rfc3339(), host));
  }

  #[test]
  fn test_syslog_line_app() {
    let ts = time::now();
    let app = "sysly";
    assert_eq!(Syslog::line(
      Facility::LOCAL0, Severity::INFO, ts, None, Some(app), None, None, "yo"),
      format!("<134>1 {} - {} - - yo", ts.rfc3339(), app));
  }

  #[test]
  fn test_syslog_line_pid() {
    let ts = time::now();
    let pid = "16";
    assert_eq!(Syslog::line(
      Facility::LOCAL0, Severity::INFO, ts, None, None, Some(pid), None, "yo"),
      format!("<134>1 {} - - {} - yo", ts.rfc3339(), pid));
  }

  #[test]
  fn test_syslog_line_msgid() {
    let ts = time::now();
    let msgid = "TCPIN";
    assert_eq!(Syslog::line(
      Facility::LOCAL0, Severity::INFO, ts, None, None, None, Some(msgid), "yo"),
      format!("<134>1 {} - - - {} yo", ts.rfc3339(), msgid));
  }

  //#[bench]
  //fn bench_assembly_line(b: &mut Bencher) {
  // b.iter(|| Syslog::line(
  //    Facility::LOCAL0, Severity::INFO, time::now(), None, None, None, None, "yo"))
  //}
}
