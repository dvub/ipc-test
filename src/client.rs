//{
fn main() -> std::io::Result<()> {
    //}
    use interprocess::local_socket::{prelude::*, GenericFilePath, GenericNamespaced, Stream};
    use std::io::{prelude::*, BufReader};

    // Pick a name.
    let name = if GenericNamespaced::is_supported() {
        "example.sock".to_ns_name::<GenericNamespaced>()?
    } else {
        "/tmp/example.sock".to_fs_name::<GenericFilePath>()?
    };

    // Preemptively allocate a sizeable buffer for receiving. This size should be enough and
    // should be easy to find for the allocator.

    // Create our connection. This will block until the server accepts our connection, but will
    // fail immediately if the server hasn't even started yet; somewhat similar to how happens
    // with TCP, where connecting to a port that's not bound to any server will send a "connection
    // refused" response, but that will take twice the ping, the roundtrip time, to reach the
    // client.
    let mut conn = Stream::connect(name)?;

    loop {
        // Limit to 5 pings/pongs
        conn.write_all(b"ping").expect("Failed to send ping");
        let mut buffer = [0; 4];
        let bytes_read = conn.read(&mut buffer).expect("Failed to read from socket");
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Client received: {}", message);
    }
    Ok(())
}
