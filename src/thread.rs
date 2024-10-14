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

    // The syncronization between the server and client, if any is used, goes here.
    // eprintln!("Server running at {printname}");
    // u32 buffer thing
    let mut buffer = [0; 4];

    let conn = listener.accept().unwrap();
    // Wrap the connection into a buffered receiver right away
    // so that we could receive a single line from it.
    let mut conn = BufReader::new(conn);

    // println!("Incoming connection!");

    // Since our client example sends first, the server should receive a line and only then
    // send a response. Otherwise, because receiving from and sending to a connection cannot
    // be simultaneous without threads or async, we can deadlock the two processes by having
    // both sides wait for the send buffer to be emptied by the other.
    conn.read_exact(&mut buffer)?;
    let incoming = u32::from_be_bytes(buffer);
    println!("Client ID: {}", incoming);

    Ok(incoming)
}

pub fn get_open_socket_name() -> Name<'static> {
    let mut open = false;
    let mut iteration = 0;
    let mut printname = format!("IPC_TEST{}.sock", iteration);

    while !open {
        printname = format!("IPC_TEST{}.sock", iteration);
        // FIX THIS CLONE
        let name = printname.clone().to_ns_name::<GenericNamespaced>().unwrap();

        open = is_socket_open(name);

        iteration += 1;
    }
    printname.clone().to_ns_name::<GenericNamespaced>().unwrap()
}

fn is_socket_open(name: Name) -> bool {
    Stream::connect(name).is_err()
}

#[cfg(test)]
mod tests {
    use super::get_open_socket_name;

    // TODO:
    // make tests actually good ?!
}
