use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

pub const NONCE_LEN: usize = 24;
pub const TAG_LEN: usize = 16;

/// Which way a packet travels. Each direction has its own key, so a packet
/// one side sent can never be accepted by that same side if reflected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    ToServer,
    ToClient,
}

/// A pair of packet keys, one per direction, sealing with XChaCha20-Poly1305.
/// A token key (`from_token`) only authenticates the handshake; each
/// attachment then gets its own keys from an X25519 exchange (`attach`), so
/// a token learned later does not open recorded sessions.
#[derive(Clone)]
pub struct Key {
    secret: [u8; 32],
    raw: ([u8; 32], [u8; 32]),
    to_server: XChaCha20Poly1305,
    to_client: XChaCha20Poly1305,
    id: u64,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Key(..)")
    }
}

fn hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part);
    }
    h.finalize().into()
}

impl Key {
    pub fn from_token(token: &str) -> Self {
        let secret = hash(&[b"rsh3 token\0", token.as_bytes()]);
        let id = hash(&[b"rsh3 key id\0", &secret]);
        let mut key = Self::from_parts(
            hash(&[b"rsh3 to server\0", &secret]),
            hash(&[b"rsh3 to client\0", &secret]),
        );
        key.secret = secret;
        key.id = u64::from_be_bytes(id[..8].try_into().expect("8 bytes")) | 1;
        key
    }

    /// Keys from their raw bytes, as `parts` gave them (live upgrade).
    pub fn from_parts(to_server: [u8; 32], to_client: [u8; 32]) -> Self {
        Self {
            secret: [0; 32],
            raw: (to_server, to_client),
            to_server: XChaCha20Poly1305::new(&to_server.into()),
            to_client: XChaCha20Poly1305::new(&to_client.into()),
            id: 0,
        }
    }

    pub fn parts(&self) -> ([u8; 32], [u8; 32]) {
        self.raw
    }

    /// The keys of one attachment: bound to this token, to the X25519
    /// shared secret and to both public keys of the exchange.
    pub fn attach(&self, shared: &[u8; 32], client_pub: &[u8; 32], server_pub: &[u8; 32]) -> Key {
        let base = hash(&[b"rsh3 attach\0", &self.secret, shared, client_pub, server_pub]);
        Self::from_parts(hash(&[b"rsh3 attach c2s\0", &base]), hash(&[b"rsh3 attach s2c\0", &base]))
    }

    /// A fingerprint that tells token keys apart without revealing them;
    /// it goes in the header so the server picks the key without trying all.
    pub fn id(&self) -> u64 {
        self.id
    }

    fn cipher(&self, dir: Dir) -> &XChaCha20Poly1305 {
        match dir {
            Dir::ToServer => &self.to_server,
            Dir::ToClient => &self.to_client,
        }
    }

    pub fn seal(&self, dir: Dir, nonce: &[u8; NONCE_LEN], aad: &[u8], plain: &[u8]) -> Vec<u8> {
        self.cipher(dir)
            .encrypt(XNonce::from_slice(nonce), Payload { msg: plain, aad })
            .expect("xchacha20poly1305 encrypt")
    }

    pub fn open(&self, dir: Dir, nonce: &[u8], aad: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
        if nonce.len() != NONCE_LEN {
            return None;
        }
        self.cipher(dir)
            .decrypt(XNonce::from_slice(nonce), Payload { msg: sealed, aad })
            .ok()
    }
}

static PREFIX: OnceLock<[u8; 16]> = OnceLock::new();
static COUNTER: AtomicU64 = AtomicU64::new(1);

/// A fresh nonce: process-random prefix and a strictly increasing counter.
/// The counter is also what the receiver uses to ignore stale packets.
pub fn next_nonce() -> ([u8; NONCE_LEN], u64) {
    let prefix = PREFIX.get_or_init(|| {
        let mut bytes = [0u8; 16];
        random_bytes(&mut bytes);
        bytes
    });
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut nonce = [0u8; NONCE_LEN];
    nonce[..16].copy_from_slice(prefix);
    nonce[16..].copy_from_slice(&counter.to_be_bytes());
    (nonce, counter)
}

/// This process's nonce prefix and next counter, for a live upgrade.
pub fn nonce_state() -> ([u8; 16], u64) {
    let (nonce, counter) = next_nonce();
    (nonce[..16].try_into().expect("prefix"), counter + 1)
}

/// Continues the nonce sequence of the process this one replaced, so peers
/// that pinned its prefix keep accepting packets and no nonce repeats.
/// Must run before the first packet is sealed.
pub fn resume_nonces(prefix: [u8; 16], counter: u64) -> bool {
    if PREFIX.set(prefix).is_err() {
        return false;
    }
    COUNTER.store(counter, Ordering::Relaxed);
    true
}

/// A fresh X25519 key pair for one handshake.
pub fn ephemeral() -> (x25519_dalek::StaticSecret, [u8; 32]) {
    let mut bytes = [0u8; 32];
    random_bytes(&mut bytes);
    let secret = x25519_dalek::StaticSecret::from(bytes);
    let public = x25519_dalek::PublicKey::from(&secret).to_bytes();
    (secret, public)
}

/// The shared secret with a peer's public key, or None for a low-order key
/// that would make the result predictable.
pub fn shared(secret: &x25519_dalek::StaticSecret, peer: &[u8; 32]) -> Option<[u8; 32]> {
    let shared = secret.diffie_hellman(&x25519_dalek::PublicKey::from(*peer));
    shared.was_contributory().then(|| shared.to_bytes())
}

pub fn random_bytes(out: &mut [u8]) {
    let mut file = std::fs::File::open("/dev/urandom").expect("/dev/urandom");
    file.read_exact(out).expect("read /dev/urandom");
}

pub fn random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    random_bytes(&mut bytes);
    u64::from_be_bytes(bytes)
}

/// A new random token for `rsh bootstrap`: 32 hex characters.
pub fn new_token() -> String {
    let mut bytes = [0u8; 16];
    random_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_and_reject_tamper() {
        let key = Key::from_token("tok");
        let (nonce, counter) = next_nonce();
        let (again, next) = next_nonce();
        assert_ne!(nonce, again);
        assert!(next > counter);
        let sealed = key.seal(Dir::ToServer, &nonce, b"hdr", b"hello");
        assert_eq!(key.open(Dir::ToServer, &nonce, b"hdr", &sealed).as_deref(), Some(&b"hello"[..]));
        assert!(key.open(Dir::ToClient, &nonce, b"hdr", &sealed).is_none(), "reflection");
        assert!(key.open(Dir::ToServer, &nonce, b"hdX", &sealed).is_none());
        assert!(Key::from_token("other").open(Dir::ToServer, &nonce, b"hdr", &sealed).is_none());
        let mut bad = sealed.clone();
        bad[0] ^= 1;
        assert!(key.open(Dir::ToServer, &nonce, b"hdr", &bad).is_none());
        assert_eq!(new_token().len(), 32);
        let (a, a_pub) = ephemeral();
        let (b, b_pub) = ephemeral();
        let shared_a = shared(&a, &b_pub).unwrap();
        assert_eq!(Some(shared_a), shared(&b, &a_pub));
        assert!(shared(&a, &[0; 32]).is_none(), "low-order point");
        let client = key.attach(&shared_a, &a_pub, &b_pub);
        let server = key.attach(&shared(&b, &a_pub).unwrap(), &a_pub, &b_pub);
        let sealed = client.seal(Dir::ToServer, &nonce, b"h", b"x");
        assert!(server.open(Dir::ToServer, &nonce, b"h", &sealed).is_some());
        assert!(key.open(Dir::ToServer, &nonce, b"h", &sealed).is_none(), "token key cannot open it");
        let other = Key::from_token("other").attach(&shared_a, &a_pub, &b_pub);
        assert!(other.open(Dir::ToServer, &nonce, b"h", &sealed).is_none(), "bound to the token");
        let (c2s, s2c) = server.parts();
        assert!(Key::from_parts(c2s, s2c).open(Dir::ToServer, &nonce, b"h", &sealed).is_some());
    }
}
