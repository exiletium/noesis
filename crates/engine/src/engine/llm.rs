//! Optional language-model bridge for free-form English.
//!
//! When no handler understands a question and a model endpoint is
//! configured, the question is sent to an OpenAI-compatible chat endpoint
//! (Ollama, llama.cpp's server, LM Studio, vLLM, or a hosted service) with a
//! description of Noesis's query language.  The model only *rewrites* the
//! question; Noesis then computes the answer itself, and the rewritten
//! query is shown so the user can check the interpretation.
//!
//! Configuration (environment variables):
//!   NOESIS_LLM_URL    base URL, e.g. http://localhost:11434/v1 (Ollama)
//!   NOESIS_LLM_MODEL  model name, e.g. qwen2.5:7b (default "default")
//!   NOESIS_LLM_KEY    bearer token, if the endpoint needs one
//!
//! An endpoint on localhost is used even in offline mode, since nothing
//! leaves the machine; any other host is used only when online data is on.

use super::*;
use std::cell::Cell;

thread_local!(static ACTIVE: Cell<bool> = const { Cell::new(false) });

struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        ACTIVE.with(|a| a.set(false));
    }
}

pub(crate) const SYSTEM_PROMPT: &str = "You translate questions into queries for Noesis, a computational engine like WolframAlpha. \
Reply with ONE line containing only the query, or NONE if the question cannot be computed. Do not answer the question yourself. \
Noesis understands: arithmetic (2^100, 100!, 355/113), 'pi to 50 digits', 'solve x^2 - 5x + 6 = 0', systems '{x + y = 3, x - y = 1}', \
'factor x^4 + 4', 'expand (x+1)^5', 'derivative of x^x', 'integrate x^2 sin(x) dx', 'integrate e^(-x^2) from -inf to inf', \
'limit sin(x)/x as x -> 0', 'sum 1/k^2, k=1 to inf', 'taylor series of e^x', ODEs like \"y'' + y = 0, y(0)=1, y'(0)=0\", \
PDEs like 'u_t = u_xx, u(0,t)=0, u(1,t)=0, u(x,0)=x(1-x)', 'plot sin(x) and cos(x)', matrices '{{1,2},{3,4}}', \
'eigenvalues of {{2,1},{1,2}}', 'mean of 1, 2, 3', 'normal distribution mean 100 sd 15 P(X > 130)', '10 miles in km', \
'speed of light', 'molar mass of caffeine', 'balance H2 + O2 -> H2O', 'pH of 0.1 M acetic acid', 'population of japan', \
'distance from Paris to London', 'time in Tokyo', 'days between 2024-01-01 and 2024-12-25', \
'compound interest 1000 at 5% for 10 years', 'mortgage 300000 at 6% for 30 years', '30% off 80', \
'kinetic energy of 2 kg at 3 m/s', 'area of a circle with radius 5', 'how many calories in a banana', 'sunrise in paris'. \
Use x as the variable unless the question names another. Write numbers as digits.";

static CONFIG: std::sync::RwLock<Option<(String, String, Option<String>)>> = std::sync::RwLock::new(None);

/// Set the model endpoint at run time (overrides the environment variables).
/// An empty URL turns the bridge off.
pub fn configure(url: &str, model: &str, key: Option<&str>) {
    let v = if url.trim().is_empty() { Some((String::new(), String::new(), None)) } else { Some((url.trim().to_string(), if model.trim().is_empty() { "default".into() } else { model.trim().to_string() }, key.map(str::to_string).filter(|k| !k.is_empty()))) };
    if let Ok(mut c) = CONFIG.write() {
        *c = v;
    }
}

/// Is a model endpoint configured and allowed right now?
fn endpoint() -> Option<(String, String, Option<String>)> {
    if let Some((url, model, key)) = CONFIG.read().ok().and_then(|c| c.clone()) {
        if url.is_empty() {
            return None;
        }
        let local = url.contains("://localhost") || url.contains("://127.0.0.1") || url.contains("://[::1]");
        if !local && !crate::domains::live::online() {
            return None;
        }
        return Some((url.trim_end_matches('/').to_string(), model, key));
    }
    let url = std::env::var("NOESIS_LLM_URL").ok().filter(|u| !u.trim().is_empty())?;
    let local = url.contains("://localhost") || url.contains("://127.0.0.1") || url.contains("://[::1]");
    if !local && !crate::domains::live::online() {
        return None;
    }
    let model = std::env::var("NOESIS_LLM_MODEL").unwrap_or_else(|_| "default".into());
    let key = std::env::var("NOESIS_LLM_KEY").ok().filter(|k| !k.is_empty());
    Some((url.trim_end_matches('/').to_string(), model, key))
}

/// Pull the query line out of a model reply.
pub(crate) fn extract(reply: &str) -> Option<String> {
    let line = reply
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("```"))?
        .trim_start_matches("Query:")
        .trim_start_matches("query:")
        .trim()
        .trim_matches(|c| c == '`' || c == '"' || c == '\'')
        .trim();
    if line.is_empty() || line.eq_ignore_ascii_case("none") || line.len() > 300 {
        return None;
    }
    Some(line.to_string())
}

#[cfg(feature = "net")]
fn ask(question: &str) -> Option<String> {
    let (url, model, key) = endpoint()?;
    let body = serde_json::json!({
        "model": model,
        "temperature": 0,
        "max_tokens": 120,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": question},
        ],
    });
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(30)).build();
    let mut req = agent.post(&format!("{url}/chat/completions")).set("Content-Type", "application/json");
    if let Some(k) = key {
        req = req.set("Authorization", &format!("Bearer {k}"));
    }
    let v: serde_json::Value = req.send_json(body).ok()?.into_json().ok()?;
    let reply = v.get("choices")?.get(0)?.get("message")?.get("content")?.as_str()?;
    extract(reply)
}

#[cfg(not(feature = "net"))]
fn ask(_question: &str) -> Option<String> {
    let _ = endpoint();
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    if ACTIVE.with(|a| a.get()) {
        return None;
    }
    ACTIVE.with(|a| a.set(true));
    let _reset = Reset;
    let rewritten = ask(q.raw.trim())?;
    if rewritten.trim().eq_ignore_ascii_case(q.raw.trim()) {
        return None;
    }
    let mut a = super::nl::again(q, &rewritten)?;
    a.pods.insert(0, pod("Rephrased", "Interpreted with the language model as", text(rewritten.clone())));
    a.assumptions.push(format!("The question was rewritten by a language model as \"{rewritten}\"; check that this is what you meant."));
    Some(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_the_query_line() {
        assert_eq!(extract("integrate x^2 dx").as_deref(), Some("integrate x^2 dx"));
        assert_eq!(extract("```\nsolve x^2 = 4\n```").as_deref(), Some("solve x^2 = 4"));
        assert_eq!(extract("Query: `10 miles in km`").as_deref(), Some("10 miles in km"));
        assert_eq!(extract("NONE"), None);
        assert_eq!(extract("   "), None);
    }

    /// A one-request OpenAI-compatible server that answers with `reply`.
    #[cfg(feature = "net")]
    fn mock(reply: &'static str) -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 65536];
            let mut got = Vec::new();
            // read headers and the JSON body
            loop {
                let n = sock.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                got.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&got);
                if let Some(h) = text.find("\r\n\r\n") {
                    let len = text[..h].lines().find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap_or(0))).unwrap_or(0);
                    if got.len() >= h + 4 + len {
                        break;
                    }
                }
            }
            assert!(String::from_utf8_lossy(&got).contains("chat/completions"));
            let body = serde_json::json!({"choices": [{"message": {"role": "assistant", "content": reply}}]}).to_string();
            let _ = write!(sock, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
        });
        format!("http://127.0.0.1:{}/v1", addr.port())
    }

    #[cfg(feature = "net")]
    #[test]
    fn rephrases_through_an_endpoint() {
        crate::domains::live::set_online(false);
        configure(&mock("compound interest 1000 at 5% for 10 years"), "mock", None);
        let r = crate::query("If I put a thousand bucks away at five percent, what do I have after a decade?");
        configure("", "", None);
        assert!(r.success, "{:?}", r.error);
        assert_eq!(r.pods[0].id, "Rephrased");
        assert_eq!(r.short_answer().as_deref(), Some("$1,628.89"));
    }
}
