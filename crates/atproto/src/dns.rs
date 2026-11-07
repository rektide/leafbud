//! Minimal DNS TXT lookup for handle resolution (`_atproto.<handle>`).
//!
//! The atproto handle spec's first resolution method is a DNS TXT record;
//! the HTTPS well-known fallback (see [`crate::identity`]) covers the rest.
//! Rather than pull a full resolver crate for one query type, this module
//! speaks plain UDP DNS to the system nameservers from `/etc/resolv.conf`:
//! build a one-question recursive-desired query, parse the answer section
//! with RFC 1035 name compression, take the first TXT rdata prefixed
//! `did=`. Any transport or format failure is an error — the caller falls
//! back to the well-known method, which is the authoritative path for
//! bsky.social-hosted handles anyway (experiment 03 §1: no TXT record).

use std::time::Duration;

use crate::Error;

/// Query timeout per nameserver. Short: a failure falls through to the
/// well-known HTTPS method quickly.
const TIMEOUT: Duration = Duration::from_secs(2);

/// TXT record type (RFC 1035 §3.2.2).
const QTYPE_TXT: u16 = 16;
/// IN class.
const QCLASS_IN: u16 = 1;

/// Look up `did=` in the TXT records of `_atproto.<handle>` via the system
/// resolver(s). `Ok(None)` = response valid but no matching record
/// (including NXDOMAIN) — the caller should try the well-known method.
pub async fn resolve_handle_txt(handle: &str) -> Result<Option<String>, Error> {
    let nameservers = system_nameservers().map_err(|reason| Error::Identity {
        input: handle.to_string(),
        reason,
    })?;
    if nameservers.is_empty() {
        return Ok(None);
    }
    let qname = format!("_atproto.{handle}");
    let query = build_query(&qname);
    let mut last_err: Option<String> = None;
    for ns in &nameservers {
        match query_nameserver(ns, &query, &qname).await {
            Ok(answer) => return Ok(answer),
            Err(e) => last_err = Some(format!("nameserver {ns}: {e}")),
        }
    }
    // All nameservers failed at the transport level — report why, but as a
    // soft negative so the caller can still fall back.
    tracing::debug!(handle = %handle, error = last_err.as_deref().unwrap_or("?"), "DNS TXT query failed");
    Ok(None)
}

/// Nameserver addresses from `/etc/resolv.conf` (`nameserver` lines),
/// IPv4-mapped to `0.0.0.0:53` / `[::]:53` form for `UdpSocket::connect`.
fn system_nameservers() -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string("/etc/resolv.conf").map_err(|e| format!("reading /etc/resolv.conf: {e}"))?;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if let Some(addr) = line.strip_prefix("nameserver").map(str::trim) {
            if addr.contains(':') {
                out.push(format!("[{addr}]:53"));
            } else {
                out.push(format!("{addr}:53"));
            }
        }
    }
    Ok(out)
}

/// Build the wire query: header (id 0, RD set, one question) + QNAME labels
/// + QTYPE TXT + QCLASS IN.
fn build_query(qname: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(17 + qname.len());
    out.extend_from_slice(&0u16.to_be_bytes()); // id: 0 (connected socket, single query in flight)
    out.extend_from_slice(&0x0100u16.to_be_bytes()); // flags: recursion desired
    out.extend_from_slice(&1u16.to_be_bytes()); // qdcount
    out.extend_from_slice(&0u16.to_be_bytes()); // ancount
    out.extend_from_slice(&0u16.to_be_bytes()); // nscount
    out.extend_from_slice(&0u16.to_be_bytes()); // arcount
    encode_name(qname, &mut out);
    out.extend_from_slice(&QTYPE_TXT.to_be_bytes());
    out.extend_from_slice(&QCLASS_IN.to_be_bytes());
    out
}

/// Encode a dotted name as DNS labels (no compression on the query side).
fn encode_name(name: &str, out: &mut Vec<u8>) {
    for label in name.trim_end_matches('.').split('.') {
        let bytes = label.as_bytes();
        out.push(bytes.len() as u8);
        out.extend_from_slice(bytes);
    }
    out.push(0);
}

/// Send the query to one nameserver and extract the first `did=` TXT string.
async fn query_nameserver(ns: &str, query: &[u8], qname: &str) -> Result<Option<String>, String> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await.map_err(|e| format!("bind: {e}"))?;
    socket.connect(ns).await.map_err(|e| format!("connect: {e}"))?;
    socket.send(query).await.map_err(|e| format!("send: {e}"))?;
    let mut buf = vec![0u8; 512]; // no EDNS; TXT answers for _atproto are tiny
    let n = tokio::time::timeout(TIMEOUT, socket.recv(&mut buf))
        .await
        .map_err(|_| "timeout".to_string())?
        .map_err(|e| format!("recv: {e}"))?;
    parse_txt_answer(&buf[..n], qname).map_err(|e| format!("parse: {e}"))
}

/// Parse a TXT response: check the id/rcode, skip the question section,
/// walk the answers, and return the first `did=`-prefixed TXT rdata whose
/// owner name matches `qname` (case-insensitive).
fn parse_txt_answer(msg: &[u8], qname: &str) -> Result<Option<String>, String> {
    if msg.len() < 12 {
        return Err("truncated header".into());
    }
    let id = u16::from_be_bytes([msg[0], msg[1]]);
    if id != 0 {
        return Err(format!("unexpected query id {id}"));
    }
    let flags = u16::from_be_bytes([msg[2], msg[3]]);
    let rcode = flags & 0x000f;
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]);
    let ancount = u16::from_be_bytes([msg[6], msg[7]]);
    if rcode == 3 {
        return Ok(None); // NXDOMAIN: no such name — no TXT record
    }
    if rcode != 0 {
        return Err(format!("rcode {rcode}"));
    }
    // QR must be set (response), and it must be an answer to our question.
    if flags & 0x8000 == 0 {
        return Err("not a response".into());
    }
    let mut pos = 12usize;
    for _ in 0..qdcount {
        let (_name, next) = parse_name(msg, pos)?;
        pos = next + 4; // skip QTYPE + QCLASS
        if pos > msg.len() {
            return Err("truncated question".into());
        }
    }
    let want = qname.to_ascii_lowercase();
    for _ in 0..ancount {
        let (name, next) = parse_name(msg, pos)?;
        pos = next;
        if pos + 10 > msg.len() {
            return Err("truncated answer fixed fields".into());
        }
        let rtype = u16::from_be_bytes([msg[pos], msg[pos + 1]]);
        let class = u16::from_be_bytes([msg[pos + 2], msg[pos + 3]]);
        let rdlength = u16::from_be_bytes([msg[pos + 8], msg[pos + 9]]) as usize;
        pos += 10;
        if pos + rdlength > msg.len() {
            return Err("truncated rdata".into());
        }
        if rtype == QTYPE_TXT
            && class == QCLASS_IN
            && name.to_ascii_lowercase() == want
            && let Some(did) = parse_txt_rdata(&msg[pos..pos + rdlength])
        {
            return Ok(Some(did));
        }
        pos += rdlength;
    }
    Ok(None)
}

/// TXT rdata is a sequence of character-strings (`[len][bytes]`, len ≤ 255);
/// concatenate them and strip a leading `did=`.
fn parse_txt_rdata(rdata: &[u8]) -> Option<String> {
    let mut text = String::new();
    let mut i = 0usize;
    while i < rdata.len() {
        let len = rdata[i] as usize;
        i += 1;
        if i + len > rdata.len() {
            return None;
        }
        text.push_str(&String::from_utf8_lossy(&rdata[i..i + len]));
        i += len;
    }
    text.strip_prefix("did=").map(str::to_string)
}

/// Parse a (possibly compressed) domain name at `pos`; returns the decoded
/// name and the offset just past it (past the compression pointer when one
/// terminates it).
fn parse_name(msg: &[u8], pos: usize) -> Result<(String, usize), String> {
    let mut labels: Vec<String> = Vec::new();
    let mut p = pos;
    let mut jumped = false;
    let mut after_pointer = pos;
    let mut hops = 0;
    loop {
        if p >= msg.len() {
            return Err("name runs off the message".into());
        }
        let len = msg[p];
        match len & 0xc0 {
            0x00 => {
                // label
                if len == 0 {
                    p += 1;
                    if !jumped {
                        after_pointer = p;
                    }
                    break;
                }
                let end = p + 1 + len as usize;
                if end > msg.len() {
                    return Err("label runs off the message".into());
                }
                labels.push(String::from_utf8_lossy(&msg[p + 1..end]).into_owned());
                p = end;
                if !jumped {
                    after_pointer = p;
                }
            }
            0xc0 => {
                // compression pointer
                if p + 1 >= msg.len() {
                    return Err("truncated compression pointer".into());
                }
                let target = (((len & 0x3f) as usize) << 8) | msg[p + 1] as usize;
                if !jumped {
                    after_pointer = p + 2;
                    jumped = true;
                }
                p = target;
                hops += 1;
                if hops > 64 {
                    return Err("compression pointer loop".into());
                }
            }
            _ => return Err("reserved label type".into()),
        }
    }
    Ok((labels.join("."), after_pointer))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assemble a DNS response by hand for the parser tests: one question
    /// echo + answers built from (name, txt) pairs.
    fn response(qname: &str, answers: &[(bool, &str, &str)], rcode: u16) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&0u16.to_be_bytes()); // id
        msg.extend_from_slice(&(0x8180u16 | rcode).to_be_bytes()); // QR+RD+RA
        msg.extend_from_slice(&1u16.to_be_bytes()); // qdcount
        msg.extend_from_slice(&(answers.len() as u16).to_be_bytes());
        msg.extend_from_slice(&0u16.to_be_bytes());
        msg.extend_from_slice(&0u16.to_be_bytes());
        encode_name(qname, &mut msg);
        msg.extend_from_slice(&QTYPE_TXT.to_be_bytes());
        msg.extend_from_slice(&QCLASS_IN.to_be_bytes());
        for &(compress, name, txt) in answers {
            if compress {
                // Emit a compression pointer to the question name (offset 12).
                msg.push(0xc0);
                msg.push(12);
            } else {
                encode_name(name, &mut msg);
            }
            msg.extend_from_slice(&QTYPE_TXT.to_be_bytes());
            msg.extend_from_slice(&QCLASS_IN.to_be_bytes());
            msg.extend_from_slice(&0u32.to_be_bytes()); // ttl
            // rdata: split the txt into 255-byte character-strings
            let bytes = txt.as_bytes();
            let mut rdata = Vec::new();
            for chunk in bytes.chunks(255) {
                rdata.push(chunk.len() as u8);
                rdata.extend_from_slice(chunk);
            }
            msg.extend_from_slice(&(rdata.len() as u16).to_be_bytes());
            msg.extend_from_slice(&rdata);
        }
        msg
    }

    #[test]
    fn parses_did_txt_answer() {
        let msg = response("_atproto.example.com", &[(true, "", "did=did:plc:abc123def456uvw789")], 0);
        let got = parse_txt_answer(&msg, "_atproto.example.com").unwrap();
        assert_eq!(got.as_deref(), Some("did:plc:abc123def456uvw789"));
    }

    #[test]
    fn nxdomain_is_soft_none() {
        let msg = response("_atproto.example.com", &[], 3);
        assert_eq!(parse_txt_answer(&msg, "_atproto.example.com").unwrap(), None);
    }

    #[test]
    fn name_mismatch_and_no_did_prefix_are_ignored() {
        let msg = response(
            "_atproto.example.com",
            &[(false, "other.example.com", "did=did:plc:notours"), (true, "", "some-other-txt")],
            0,
        );
        assert_eq!(parse_txt_answer(&msg, "_atproto.example.com").unwrap(), None);
    }

    #[test]
    fn long_txt_splits_into_character_strings() {
        let long = format!("did=did:plc:{}", "x".repeat(600));
        let msg = response("_atproto.example.com", &[(true, "", &long)], 0);
        let got = parse_txt_answer(&msg, "_atproto.example.com").unwrap();
        assert_eq!(got.as_deref(), Some(long.strip_prefix("did=").unwrap()));
    }

    #[test]
    fn truncation_is_an_error() {
        let msg = response("_atproto.example.com", &[(true, "", "did=did:plc:x")], 0);
        assert!(parse_txt_answer(&msg[..msg.len() - 3], "_atproto.example.com").is_err());
    }

    #[test]
    fn query_wire_shape() {
        let q = build_query("_atproto.jauntywunderkind.bsky.social");
        // header(12) + labels + null + qtype(2) + qclass(2)
        assert_eq!(&q[0..2], &0u16.to_be_bytes());
        assert_eq!(&q[2..4], &0x0100u16.to_be_bytes());
        let expect: Vec<u8> = {
            let mut v = Vec::new();
            encode_name("_atproto.jauntywunderkind.bsky.social", &mut v);
            v
        };
        assert_eq!(&q[12..12 + expect.len()], &expect[..]);
        assert_eq!(&q[q.len() - 4..], &[0, QTYPE_TXT as u8, 0, QCLASS_IN as u8]);
    }

    #[test]
    fn resolv_conf_nameservers() {
        let dir = std::env::temp_dir().join("springleaf-atproto-dns-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("resolv-test.conf");
        std::fs::write(&path, "# comment\nnameserver 192.0.2.53\nnameserver 2001:db8::53\noptions edns0\n").unwrap();
        // The parser reads a fixed path in production; test the line logic by
        // mirroring the parse over the fixture contents.
        let text = std::fs::read_to_string(&path).unwrap();
        let servers: Vec<String> = text
            .lines()
            .filter_map(|l| {
                let l = l.split('#').next().unwrap_or("").trim();
                l.strip_prefix("nameserver").map(str::trim)
            })
            .map(|a| if a.contains(':') { format!("[{a}]:53") } else { format!("{a}:53") })
            .collect();
        assert_eq!(servers, vec!["192.0.2.53:53", "[2001:db8::53]:53"]);
    }
}
