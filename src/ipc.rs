use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Name, Stream};
use std::io::{self, prelude::*, BufReader};

pub fn listen_for_client_id(name: Name) -> anyhow::Result<u32> {
    // Configure our listener...
    let opts = ListenerOptions::new().name(name);
    // ...then create it.
    let listener = match opts.create_sync() {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            // When a program that uses a file-type socket name terminates its socket server
            // without deleting the file, a "corpse socket" remains, which can neither be
            // connected to nor reused by a new listener. Normally, Interprocess takes care of
            // this on affected platforms by deleting the socket file when the listener is
            // dropped. (This is vulnerable to all sorts of races and thus can be disabled.)
            //
            // There are multiple ways this error can be handled, if it occurs, but when the
            // listener only comes from Interprocess, it can be assumed that its previous instance
            // either has crashed or simply hasn't exited yet. In this example, we leave cleanup
            // up to the user, but in a real application, you usually don't want to do that.
            eprintln!(
                "Error: could not start server because the socket file is occupied. Please check if
                        is in use by another process and try again."
            );
            return Err(e.into());
        }
        x => x?,
    };

    // u32 buffer thing
    let mut buffer = [0; 4];

    let conn = listener.accept().unwrap();
    let mut conn = BufReader::new(conn);

    conn.read_exact(&mut buffer)?;
    let incoming = u32::from_be_bytes(buffer);

    println!("Client ID: {}", incoming);

    Ok(incoming)
}

pub fn get_open_socket_name(prefix: &str) -> io::Result<Name<'static>> {
    let mut open = false;
    let mut iteration = 0;
    let mut printname = String::new();

    while !open {
        printname = format!("{}{}.sock", prefix, iteration);
        // FIX THIS CLONE
        let name = printname.clone().to_ns_name::<GenericNamespaced>()?;

        open = is_socket_open(name);

        iteration += 1;
    }
    Ok(printname.clone().to_ns_name::<GenericNamespaced>().unwrap())
}

fn is_socket_open(name: Name) -> bool {
    Stream::connect(name).is_err()
}

#[cfg(test)]
mod tests {
    // TODO: tests are okay, but they could probably be rewritten to be cleaner.

    use super::get_open_socket_name;
    use super::is_socket_open;
    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};

    #[test]
    fn open_socket() {
        // any silly name will do here :3
        let name = "TEST_SOCKET0_".to_ns_name::<GenericNamespaced>().unwrap();
        assert!(is_socket_open(name.clone()));
    }

    #[test]
    fn occupied_socket() {
        let name = "TEST_SOCKET00_".to_ns_name::<GenericNamespaced>().unwrap();

        let opts = ListenerOptions::new().name(name.clone());
        let _listener = opts.create_sync().unwrap();

        assert!(!is_socket_open(name.clone()));
    }

    #[test]
    fn get_first_open_name() {
        let prefix = "TEST_SOCKET1_";

        let output = get_open_socket_name(prefix).unwrap();
        let expected = "TEST_SOCKET1_0.sock"
            .to_ns_name::<GenericNamespaced>()
            .unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn get_second_open_name() {
        let prefix = "TEST_SOCKET2_";

        let opts = ListenerOptions::new().name(
            "TEST_SOCKET2_0.sock"
                .to_ns_name::<GenericNamespaced>()
                .unwrap(),
        );
        let _listener = opts.create_sync().unwrap();

        let output = get_open_socket_name(prefix).unwrap();
        let expected = "TEST_SOCKET2_1.sock"
            .to_ns_name::<GenericNamespaced>()
            .unwrap();

        assert_eq!(output, expected);
    }
}
