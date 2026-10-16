//! The receive path (RCV-00..11): one streamed poll at a time, liveness, host failover.

use {
    super::Inner,
    crate::{protocol::constants::FAST_FAILURE_MS, rt},
    futures::{FutureExt, StreamExt, channel::mpsc},
    serde_json::{Value, json},
    std::{
        collections::HashSet,
        sync::{Arc, atomic::Ordering},
    },
};

pub(crate) fn start(inner: &Arc<Inner>) {
    let (tx, rx) = mpsc::unbounded();
    let generation = {
        let mut p = inner.poll.lock().expect("lock");
        if p.running {
            return;
        }
        p.running = true;
        p.generation += 1;
        p.kick = Some(tx);
        p.generation
    };
    inner.superseded.store(false, Ordering::SeqCst);
    let me = inner.clone();
    rt::spawn(async move {
        me.flush_outbox().await;
        run(me, generation, rx).await;
    });
}

pub(crate) fn stop(inner: &Inner) {
    let mut p = inner.poll.lock().expect("lock");
    p.running = false;
    p.generation += 1;
    p.kick = None;
}

pub(crate) fn kick(inner: &Inner) {
    if let Some(k) = &inner.poll.lock().expect("lock").kick {
        let _ = k.unbounded_send(());
    }
}

fn current(inner: &Inner, generation: u64) -> bool {
    let p = inner.poll.lock().expect("lock");
    p.running && p.generation == generation
}

enum Outcome {
    Last(Value),
    Http { status: u16, retry_after_s: Option<u64> },
    Dead,
    Cut { heartbeat_seen: bool },
    Kicked,
}

async fn run(
    inner: Arc<Inner>,
    generation: u64,
    mut kick: mpsc::UnboundedReceiver<()>,
) {
    let mut tried: HashSet<usize> = HashSet::new();
    let mut cut_retried = false;
    let mut jwt_retried = false;
    let mut failures: u32 = 0;
    // RCV-08: the first poll after a start goes to the primary host.
    inner.relay.set_host_index(0);
    while current(&inner, generation) {
        let started = rt::now_ms();
        let outcome = rt::boxed(poll_once(&inner, &mut kick)).await;
        if !current(&inner, generation) {
            break;
        }
        match outcome {
            Outcome::Last(last) => {
                tried.clear();
                cut_retried = false;
                jwt_retried = false;
                failures = 0;
                inner.stats.polls.fetch_add(1, Ordering::Relaxed);
                match rt::boxed(handle_last(&inner, last)).await {
                    Ok(true) => continue,
                    Ok(false) => break,
                    Err(_) => {
                        // A failed handle or cursor write: the next poll redelivers, which seq dedupe absorbs.
                        failures += 1;
                        sleep_or_kick(
                            inner.relay.backoff_ms(failures - 1),
                            &mut kick,
                        )
                        .await;
                    }
                }
            }
            Outcome::Kicked => {
                if inner.liveness_dead.swap(false, Ordering::SeqCst) {
                    failures += 1;
                    next_host(&inner, &mut tried, started, failures, &mut kick)
                        .await;
                }
            }
            Outcome::Cut { heartbeat_seen } => {
                // Retry once on the same host, at once; a stream that proved healthy re-arms it.
                if heartbeat_seen {
                    cut_retried = false;
                }
                if !cut_retried {
                    cut_retried = true;
                    continue;
                }
                cut_retried = false;
                failures += 1;
                next_host(&inner, &mut tried, started, failures, &mut kick)
                    .await;
            }
            Outcome::Http { status, retry_after_s } => {
                failures += 1;
                if status == 401 && !jwt_retried {
                    jwt_retried = true;
                    inner.relay.jwt(true);
                    continue;
                }
                // RCV-10, first match wins.
                if let Some(s) = retry_after_s {
                    sleep_or_kick(s * 1000, &mut kick).await;
                } else if matches!(status, 401 | 403 | 429) {
                    sleep_or_kick(
                        inner.relay.backoff_ms(failures - 1),
                        &mut kick,
                    )
                    .await;
                } else if status < 500 {
                    sleep_or_kick(
                        inner.relay.config().backoff.max_ms,
                        &mut kick,
                    )
                    .await;
                } else {
                    next_host(&inner, &mut tried, started, failures, &mut kick)
                        .await;
                }
            }
            Outcome::Dead => {
                failures += 1;
                next_host(&inner, &mut tried, started, failures, &mut kick)
                    .await;
            }
        }
    }
}

/// Returns false when polling must stop (superseded).
async fn handle_last(
    inner: &Arc<Inner>,
    last: Value,
) -> Result<bool, super::SignError> {
    match last.get("end").and_then(Value::as_str) {
        Some("superseded") => {
            // RCV-04: stop. Native resumes on foreground; web waits for the next call.
            inner.superseded.store(true, Ordering::SeqCst);
            stop(inner);
            return Ok(false);
        }
        Some("cursor_expired") => {
            inner.on_cursor_expired().await?;
            inner.state.lock().await.cursor = None;
            inner.store.remove("poll/cursor").await?;
            return Ok(true);
        }
        _ => {}
    }
    let events = last
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !events.is_empty() {
        inner.handle_events(events).await?;
    }
    // RCV-03: persist the cursor only after handling its events. It is opaque, and absent
    // until there is something to acknowledge.
    if let Some(cursor) = last["cursor"].as_str() {
        inner.store.put("poll/cursor", &cursor).await?;
        inner.state.lock().await.cursor = Some(cursor.into());
    }
    inner.sweep_expired().await?;
    let delay = inner.relay.config().repoll_delay_ms;
    if delay > 0 {
        rt::sleep_ms(delay).await;
    }
    Ok(true)
}

async fn sleep_or_kick(ms: u64, kick: &mut mpsc::UnboundedReceiver<()>) {
    let s = rt::sleep_ms(ms).fuse();
    futures::pin_mut!(s);
    futures::select! {
        _ = s => {},
        _ = kick.next() => {},
    }
}

/// RCV-10 rule 5.
async fn next_host(
    inner: &Inner,
    tried: &mut HashSet<usize>,
    started: u64,
    failures: u32,
    kick: &mut mpsc::UnboundedReceiver<()>,
) {
    tried.insert(inner.relay.host_index());
    let hosts = inner.relay.hosts();
    match (0..hosts.len()).find(|i| !tried.contains(i)) {
        None => {
            tried.clear();
            sleep_or_kick(
                inner.relay.backoff_ms(failures.saturating_sub(1)),
                kick,
            )
            .await;
            inner.relay.set_host_index(0);
        }
        Some(i) => {
            if rt::now_ms().saturating_sub(started) < FAST_FAILURE_MS {
                sleep_or_kick(
                    inner.relay.backoff_ms(failures.saturating_sub(1)),
                    kick,
                )
                .await;
            }
            inner.relay.set_host_index(i);
        }
    }
}

async fn poll_once(
    inner: &Arc<Inner>,
    kick: &mut mpsc::UnboundedReceiver<()>,
) -> Outcome {
    inner.first_line_seen.store(false, Ordering::SeqCst);
    let sent = rt::now_ms();
    inner.last_byte_ms.store(sent, Ordering::SeqCst);
    let (body, host) = {
        let mut st = inner.state.lock().await;
        // RCV-07: n grows across polls, restarts and tabs.
        st.n = rt::now_ms().max(st.n + 1);
        if inner.store.put("poll/n", &st.n).await.is_err() {
            return Outcome::Dead;
        }
        let mut b = json!({ "n": st.n });
        if let Some(c) = &st.cursor {
            b["cursor"] = json!(c);
        }
        (serde_json::to_vec(&b).expect("json"), inner.relay.host())
    };
    let first_line_timeout = inner.cfg_first_line_timeout();
    let req = inner.relay.poll_request(&host, body);
    let open = inner.relay.http.stream(req).fuse();
    let open_deadline = rt::sleep_ms(first_line_timeout).fuse();
    futures::pin_mut!(open, open_deadline);
    let res = futures::select! {
        r = open => r,
        _ = open_deadline => return Outcome::Dead,
        _ = kick.next() => return Outcome::Kicked,
    };
    let res = match res {
        Ok(r) => r,
        Err(_) => return Outcome::Dead,
    };
    if !(200..300).contains(&res.status) {
        return Outcome::Http {
            status: res.status,
            retry_after_s: res.retry_after_s,
        };
    }
    let mut body = res.body;
    let mut buf: Vec<u8> = Vec::new();
    let mut first = true;
    let mut heartbeat_seen = false;
    loop {
        let wait = if first {
            first_line_timeout.saturating_sub(rt::now_ms().saturating_sub(sent))
        } else {
            inner.relay.config().foreground_stale_ms
        };
        let next = body.next().fuse();
        let silence = rt::sleep_ms(wait).fuse();
        futures::pin_mut!(next, silence);
        let chunk = futures::select! {
            c = next => c,
            _ = silence => return Outcome::Dead,
            _ = kick.next() => return Outcome::Kicked,
        };
        let chunk = match chunk {
            None => {
                return if first {
                    Outcome::Dead
                } else {
                    Outcome::Cut { heartbeat_seen }
                };
            }
            Some(Err(_)) => {
                return if first {
                    Outcome::Dead
                } else {
                    Outcome::Cut { heartbeat_seen }
                };
            }
            Some(Ok(c)) => c,
        };
        inner.last_byte_ms.store(rt::now_ms(), Ordering::SeqCst);
        buf.extend_from_slice(&chunk);
        while let Some(nl) = buf.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = buf.drain(..=nl).collect();
            let Some(v) =
                crate::protocol::ijson::value(&line[..line.len() - 1])
            else {
                continue;
            };
            if let Some(t) = v.get("t").and_then(Value::as_u64) {
                inner.on_relay_time(t);
            }
            if first {
                first = false;
                inner.first_line_seen.store(true, Ordering::SeqCst);
                inner.stats.first_lines.fetch_add(1, Ordering::Relaxed);
                if let Some(cfg) = v.get("config") {
                    inner.relay.apply_config(cfg);
                }
                continue;
            }
            if v.get("events").is_some() {
                return Outcome::Last(v);
            }
            heartbeat_seen = true;
            // A held poll has nothing left to deliver, so a heartbeat means caught up (EVT-06).
            let _ = inner.sweep_expired().await;
        }
    }
}
