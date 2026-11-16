//! The query engine: takes free-form input, decides what it means, and
//! produces WolframAlpha-style result pods.

pub mod math;
pub mod commands;
pub mod ode;
pub mod explain;
pub mod nl;
pub mod nl_math;
pub mod nl_more;
pub mod pde;
pub mod advanced;
pub mod course;
pub mod llm;
pub mod rewrite;
pub mod sequences;
pub mod analysis;
pub mod toolbox;
pub mod methods;
pub mod systems;
pub mod computing;
pub mod inference;
pub mod numbers;
pub mod daily;
pub mod school2;
pub mod ccss;
pub mod chance;
pub mod intseq;
pub mod graphs;
pub mod physics2;
pub mod geology;
pub mod cfrac;
pub mod math3;
pub mod mathfacts;

use crate::budget;
use crate::expr::Expr;
use crate::plot::Plot;
use crate::print::{self, MathNode};
use serde::Serialize;
use std::time::Instant;

// ---------------------------------------------------------------------------
// result model
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Content {
    /// Typeset mathematics, with plain-text and LaTeX renderings.
    Math { node: MathNode, text: String, latex: String },
    Text { text: String },
    Table { header: Option<Vec<String>>, rows: Vec<Vec<Content>> },
    Plot { plot: Box<Plot> },
    Steps { steps: Vec<StepOut> },
}

#[derive(Clone, Debug, Serialize)]
pub struct StepOut {
    pub text: String,
    pub math: Option<Box<Content>>,
    pub depth: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Subpod {
    pub title: String,
    pub content: Content,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pod {
    pub id: String,
    pub title: String,
    pub subpods: Vec<Subpod>,
    /// The main answer.
    pub primary: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueryResult {
    pub query: String,
    pub success: bool,
    /// How the input was understood (shown as the "Input" pod as well).
    pub interpretation: Option<String>,
    pub pods: Vec<Pod>,
    pub assumptions: Vec<String>,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
    pub timing_ms: f64,
    pub device: String,
}

impl QueryResult {
    /// The primary answer as plain text.
    pub fn short_answer(&self) -> Option<String> {
        let pod = self.pods.iter().find(|p| p.primary).or_else(|| self.pods.iter().find(|p| p.id != "Input"))?;
        let sp = pod.subpods.first()?;
        Some(content_text(&sp.content))
    }
}

pub fn content_text(c: &Content) -> String {
    match c {
        Content::Math { text, .. } => text.clone(),
        Content::Text { text } => text.clone(),
        Content::Table { header, rows } => {
            let mut out = String::new();
            if let Some(h) = header {
                out += &h.join(" | ");
                out += "\n";
            }
            for r in rows {
                out += &r.iter().map(content_text).collect::<Vec<_>>().join(" | ");
                out += "\n";
            }
            out.trim_end().to_string()
        }
        Content::Plot { plot } => format!("[plot: {}]", plot.exprs.join(", ")),
        Content::Steps { steps } => steps.iter().map(|s| format!("{}{}{}", "  ".repeat(s.depth), s.text, s.math.as_ref().map(|m| format!(" {}", content_text(m))).unwrap_or_default())).collect::<Vec<_>>().join("\n"),
    }
}

// ---------------------------------------------------------------------------
// content helpers
// ---------------------------------------------------------------------------

pub fn math(e: &Expr) -> Content {
    math_digits(e, 6)
}

pub fn math_digits(e: &Expr, digits: usize) -> Content {
    let fmt = print::Fmt { digits, ..Default::default() };
    let node = print::node_with(e, &fmt);
    Content::Math { text: print::text_with(e, &fmt), latex: print::node_latex(&node), node }
}

pub fn math_d(d: &print::D) -> Content {
    let node = print::d_node(d);
    Content::Math { text: print::d_text(d), latex: print::node_latex(&node), node }
}

pub fn text(s: impl Into<String>) -> Content {
    Content::Text { text: s.into() }
}

pub fn table(header: Option<Vec<&str>>, rows: Vec<Vec<Content>>) -> Content {
    Content::Table { header: header.map(|h| h.into_iter().map(String::from).collect()), rows }
}

pub fn plot(p: Plot) -> Content {
    Content::Plot { plot: Box::new(p) }
}

pub fn pod(id: &str, title: &str, content: Content) -> Pod {
    Pod { id: id.into(), title: title.into(), subpods: vec![Subpod { title: String::new(), content }], primary: false }
}

pub fn pod_multi(id: &str, title: &str, subs: Vec<(String, Content)>) -> Pod {
    Pod { id: id.into(), title: title.into(), subpods: subs.into_iter().map(|(t, c)| Subpod { title: t, content: c }).collect(), primary: false }
}

pub fn primary(mut p: Pod) -> Pod {
    p.primary = true;
    p
}

pub fn steps_content(steps: &[crate::steps::Step]) -> Content {
    Content::Steps {
        steps: steps
            .iter()
            .map(|s| StepOut { text: s.text.clone(), math: s.math.as_ref().map(|m| Box::new(math(m))), depth: s.depth })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// options and context
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Options {
    /// Overall time limit in seconds.
    pub timeout: f64,
    /// Produce step-by-step solutions where available.
    pub steps: bool,
    /// Significant digits for decimal approximations.
    pub digits: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options { timeout: 20.0, steps: true, digits: 6 }
    }
}

/// What a handler sees.
pub struct Query<'a> {
    /// The input as typed.
    pub raw: &'a str,
    /// Cleaned: trimmed, filler phrases removed, unicode normalised.
    pub clean: String,
    /// `clean` in lower case, for keyword matching.
    pub lower: String,
    pub opts: &'a Options,
}

/// A handler either claims the query (returning pods and an interpretation)
/// or passes.
pub struct Answer {
    pub interpretation: String,
    pub pods: Vec<Pod>,
    pub assumptions: Vec<String>,
}

impl Answer {
    pub fn new(interpretation: impl Into<String>, pods: Vec<Pod>) -> Answer {
        Answer { interpretation: interpretation.into(), pods, assumptions: vec![] }
    }
}

pub type Handler = fn(&Query) -> Option<Answer>;

/// Handlers in priority order.  Domain handlers recognise their own
/// vocabulary; the general math analyzer comes last.
fn handlers() -> Vec<(&'static str, Handler)> {
    let mut v: Vec<(&'static str, Handler)> = Vec::new();
    v.push(("pde", pde::handle));
    v.push(("systems", systems::handle));
    v.push(("graphs", graphs::handle));
    v.push(("ccss", ccss::handle));
    v.push(("computing", computing::handle));
    v.push(("numbers", numbers::handle));
    v.push(("daily", daily::handle));
    v.push(("chance", chance::handle));
    v.push(("physics2", physics2::handle));
    v.push(("geology", geology::handle));
    v.push(("cfrac", cfrac::handle));
    v.push(("words", crate::domains::lexicon::handle));
    v.push(("facts", crate::domains::facts::handle));
    v.push(("language", nl::handle));
    v.push(("methods", methods::handle));
    v.push(("ode", ode::handle));
    v.push(("sequences", sequences::handle));
    v.extend(crate::domains::handlers());
    v.push(("analysis", analysis::handle));
    v.push(("toolbox", toolbox::handle));
    v.push(("inference", inference::handle));
    v.push(("school", school2::handle));
    v.push(("math3", math3::handle));
    v.push(("mathfacts", mathfacts::handle));
    v.push(("commands", commands::handle));
    v.push(("intseq", intseq::handle));
    v.push(("math", math::handle));
    v.push(("rephrase", llm::handle));
    v.push(("encyclopedia", crate::domains::live::fallback));
    v
}

// ---------------------------------------------------------------------------
// query cleaning
// ---------------------------------------------------------------------------

const FILLER_PREFIXES: &[&str] = &[
    "what is the value of ",
    "what is the ",
    "what is ",
    "what are the ",
    "what are ",
    "what's the ",
    "what's ",
    "whats ",
    "how much is ",
    "how many ",
    "calculate the ",
    "calculate ",
    "compute the ",
    "compute ",
    "evaluate the ",
    "evaluate ",
    "find the ",
    "find ",
    "determine the ",
    "determine ",
    "show me the ",
    "show me ",
    "show ",
    "give me the ",
    "give me ",
    "tell me the ",
    "tell me ",
    "please ",
    "can you ",
    "could you ",
    "i want to ",
    "i need to ",
];

pub fn clean_query(s: &str) -> String {
    let mut q = s.trim().to_string();
    // unify dashes and quotes
    q = q.replace(['\u{2212}', '\u{2013}', '\u{2014}'], "-").replace(['\u{2018}', '\u{2019}'], "'").replace(['\u{201c}', '\u{201d}'], "\"");
    loop {
        let lower = q.to_lowercase();
        let mut changed = false;
        for p in FILLER_PREFIXES {
            if lower.starts_with(p) && q.len() > p.len() {
                q = q[p.len()..].trim_start().to_string();
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    // "1+1=" as typed into a calculator: evaluate the left side
    if q.ends_with('=') && !q.ends_with("==") && !q.ends_with("<=") && !q.ends_with(">=") && !q.ends_with("!=") && q.len() > 1 {
        q.pop();
        q = q.trim_end().to_string();
    }
    for suf in ["?", " please", "."] {
        while q.to_lowercase().ends_with(suf) && q.len() > suf.len() {
            q.truncate(q.len() - suf.len());
            q = q.trim_end().to_string();
        }
    }
    q
}

// ---------------------------------------------------------------------------
// running a query
// ---------------------------------------------------------------------------

/// Evaluate a query with default options.
pub fn query(input: &str) -> QueryResult {
    query_with(input, &Options::default())
}

pub fn query_with(input: &str, opts: &Options) -> QueryResult {
    let started = Instant::now();
    let input_owned = input.to_string();
    let opts2 = opts.clone();
    let outcome = budget::run_isolated(opts.timeout, move || run_handlers(&input_owned, &opts2));
    let mut res = match outcome {
        Some(r) => r,
        None => QueryResult {
            query: input.to_string(),
            success: false,
            interpretation: None,
            pods: vec![],
            assumptions: vec![],
            suggestions: vec![],
            error: Some("The computation ran out of time.".into()),
            timing_ms: 0.0,
            device: String::new(),
        },
    };
    res.timing_ms = started.elapsed().as_secs_f64() * 1000.0;
    res.device = crate::compute::device_label();
    res
}

fn run_handlers(input: &str, opts: &Options) -> QueryResult {
    let clean = clean_query(input);
    let q = Query { raw: input, lower: clean.to_lowercase(), clean, opts };
    let mut res = QueryResult {
        query: input.to_string(),
        success: false,
        interpretation: None,
        pods: vec![],
        assumptions: vec![],
        suggestions: vec![],
        error: None,
        timing_ms: 0.0,
        device: String::new(),
    };
    if q.clean.is_empty() {
        res.error = Some("Enter a question, a calculation or a formula.".into());
        return res;
    }
    let all = handlers();
    // the fallbacks (language model, encyclopedia) run only after rewordings
    let (main, fallback): (Vec<_>, Vec<_>) = all.into_iter().partition(|(n, _)| !matches!(*n, "rephrase" | "encyclopedia"));
    let try_all = |q: &Query, hs: &[(&'static str, Handler)]| -> Option<Answer> {
        for (_name, h) in hs {
            if let Some(ans) = budget::attempt_opt(budget::remaining(), || h(q)) {
                if !ans.pods.is_empty() {
                    return Some(ans);
                }
            }
            if budget::remaining() <= 0.05 {
                break;
            }
        }
        None
    };
    let mut found = try_all(&q, &main);
    if found.is_none() {
        for cand in rewrite::candidates(&q.clean).into_iter().take(12) {
            if budget::remaining() <= 0.2 {
                break;
            }
            let q2 = Query { raw: input, lower: cand.to_lowercase(), clean: cand.clone(), opts };
            if let Some(mut ans) = try_all(&q2, &main) {
                ans.assumptions.insert(0, format!("Interpreted as: {cand}"));
                found = Some(ans);
                break;
            }
        }
    }
    if found.is_none() {
        found = try_all(&q, &fallback);
    }
    if let Some(ans) = found {
        res.success = true;
        res.interpretation = Some(ans.interpretation);
        res.pods = ans.pods;
        res.assumptions = ans.assumptions;
        return res;
    }
    res.error = Some("Noesis did not understand this input.".into());
    res.suggestions = suggestions(&q.lower);
    res
}

fn suggestions(lower: &str) -> Vec<String> {
    let mut s = vec![];
    if lower.contains("integral") || lower.contains("integrate") {
        s.push("integrate x^2 sin(x) dx".into());
    }
    if lower.contains("deriv") {
        s.push("derivative of x^3 ln(x)".into());
    }
    if s.is_empty() {
        s.extend(["solve x^2 - 5x + 6 = 0", "integrate e^(-x^2) from -inf to inf", "plot sin(x)/x", "10 miles in km", "molar mass of caffeine"].iter().map(|x| x.to_string()));
    }
    s
}

// ---------------------------------------------------------------------------
// parallel pod evaluation
// ---------------------------------------------------------------------------

pub type PodJob = Box<dyn FnOnce() -> Option<Vec<Pod>> + Send>;

fn pool() -> &'static rayon::ThreadPool {
    static P: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    P.get_or_init(|| rayon::ThreadPoolBuilder::new().stack_size(256 << 20).thread_name(|i| format!("noesis-pod-{i}")).build().expect("thread pool"))
}

/// Run pod builders in parallel, each with its own share of the time
/// budget; results keep the order of `jobs`.  A job that fails or times out
/// simply contributes no pods.
pub fn run_jobs(jobs: Vec<PodJob>) -> Vec<Pod> {
    let left = budget::remaining();
    let per = (left * 0.85).max(0.2);
    let deadline = Instant::now() + std::time::Duration::from_secs_f64(per);
    let mut slots: Vec<Option<Vec<Pod>>> = (0..jobs.len()).map(|_| None).collect();
    pool().scope(|s| {
        for (slot, job) in slots.iter_mut().zip(jobs) {
            s.spawn(move |_| {
                let secs = deadline.saturating_duration_since(Instant::now()).as_secs_f64();
                if secs > 0.01 {
                    *slot = budget::attempt_opt(secs, job);
                }
            });
        }
    });
    slots.into_iter().flatten().flatten().collect()
}

/// Build a job from a closure.
pub fn job(f: impl FnOnce() -> Option<Vec<Pod>> + Send + 'static) -> PodJob {
    Box::new(f)
}

pub fn job1(f: impl FnOnce() -> Option<Pod> + Send + 'static) -> PodJob {
    Box::new(move || f().map(|p| vec![p]))
}
