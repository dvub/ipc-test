use crate::{PluginParams, SerializableParams};

use serde_json::json;

use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
use std::io::{self, prelude::*};
use std::sync::Arc;
use std::thread::spawn;

pub fn ipc_server_listener(params_clone: Arc<PluginParams>) {
    // a thread appears!
    spawn(move || {
        // Define a function that checks for errors in incoming connections. We'll use this to filter
        // through connections that fail on initialization for one reason or another.
        fn handle_error(conn: io::Result<Stream>) -> Option<Stream> {
            match conn {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("Incoming connection failed: {e}");
                    None
                }
            }
        }

        // Pick a name.
        let printname = "example.sock";
        let name = printname.to_ns_name::<GenericNamespaced>()?;

        // Configure our listener...
        let opts = ListenerOptions::new().name(name);

        let mut buffer = [0; 128];

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
                        {printname} is in use by another process and try again."
            );
                return Err(e);
            }
            x => x?,
        };

        // The syncronization between the server and client, if any is used, goes here.
        eprintln!("Server running at {printname}");

        if let Some(mut conn) = listener.incoming().filter_map(handle_error).next() {
            loop {
                let num_bytes_read = conn.read(&mut buffer).expect("Failed to read from socket");

                let message = String::from_utf8(buffer[..num_bytes_read].to_vec()).unwrap();
                // println!("Server received: {}", message);
                // println!("Sending a new message..");

                let gain = params_clone.gain.value();
                let serializable_params = SerializableParams { gain };

                let json_string = json!(serializable_params).to_string();
                let message_as_bytes = json_string.as_bytes();
                conn.write_all(message_as_bytes)
                    .expect("Failed to write to socket");

                // TODO:
                // is this necessary?
                buffer = [0; 128];
            }
        }
        Ok(())
    });
}
