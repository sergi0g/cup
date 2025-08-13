use crate::socket::Socket;

/// Wraps a Docker client and provides useful functions get data from it.
pub struct Docker {
    socket: Socket,
    client: bollard::Docker
}

impl Docker {
}
