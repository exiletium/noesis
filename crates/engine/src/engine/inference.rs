//! Statistical inference from named parameters in any order:
//! "t-test mu0=250, xbar=264, s=18, n=60", "test for binomial parameter
//! p0=0.75, p-hat=0.84, n=40", "binomial confidence interval n=120,
//! p-hat=0.42", "CI for variance, s^2=21.4, n=300", "sample size sigma=12.5,
//! margin of error .03", and the formula behind each test or interval when
//! no numbers are given ("two proportion hypothesis test", "Jeffreys
//! interval formula").

use super::*;
use regex::Regex;
use statrs::distribution::{Beta, ChiSquared, ContinuousCDF, Normal, StudentsT};
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn fmt(v: f64) -> String {
    crate::stats::fmt(v)
}

fn params(l: &str) -> HashMap<String, f64> {
    let mut out = HashMap::new();
    let alias = |k: &str| -> Option<&'static str> {
        Some(match k {
            "mu0" | "μ0" | "mu_0" | "μ₀" | "mu" | "μ" | "m0" => "mu0",
            "xbar" | "x-bar" | "x̄" | "mean" | "sample mean" | "xbar1" | "x1" | "xbar_1" => "xbar",
            "xbar2" | "x2" | "xbar_2" => "xbar2",
            "s" | "sd" | "s1" | "standard deviation" | "sample standard deviation" => "s",
            "s2" | "s_2" => "s2",
            "sigma" | "σ" | "population standard deviation" => "sigma",
            "s^2" | "s²" | "variance" | "sample variance" => "var",
            "n" | "n1" | "sample size" | "n_1" => "n",
            "n2" | "n_2" => "n2",
            "p0" | "p_0" | "p₀" => "p0",
            "p-hat" | "phat" | "p̂" | "p" | "p1" | "phat1" | "p-hat1" | "sample proportion" => "phat",
            "p2" | "phat2" | "p-hat2" => "phat2",
            "x" | "successes" | "k" => "x",
            "margin of error" | "e" | "me" | "margin" | "error" => "e",
            "alpha" | "α" | "significance level" | "significance" => "alpha",
            "confidence level" | "confidence" | "cl" | "level" => "level",
            _ => return None,
        })
    };
    for m in re!(r"(?i)(mu_?0|μ_?0|μ₀|m0|x-?bar_?[12]?|x̄|p-?hat_?[12]?|p̂|p_?0|p₀|s\^2|s²|sigma|σ|alpha|α|margin of error|sample mean|sample standard deviation|population standard deviation|sample variance|sample proportion|sample size|standard deviation|variance|confidence level|significance level|confidence|successes|mean|\bn_?[12]?\b|\bs_?[12]?\b|\bp[12]?\b|\bx[12]?\b|\bmu\b|\bμ\b|\bk\b|\be\b|\bme\b)\s*(?:=|:|of|is)?\s*(-?\.?\d+(?:\.\d+)?)\s*(%)?").captures_iter(l) {
        let key = m[1].to_lowercase().replace(' ', " ");
        let Some(k) = alias(key.as_str()) else { continue };
        let Ok(mut v) = m[2].parse::<f64>().or_else(|_| format!("0{}", &m[2]).parse::<f64>()) else { continue };
        if m.get(3).is_some() {
            v /= 100.0;
        }
        out.entry(k.to_string()).or_insert(v);
    }
    // "95% confidence" without a key
    if !out.contains_key("level") {
        if let Some(m) = re!(r"(\d+(?:\.\d+)?)\s*%\s*(?:confidence|ci\b|level)").captures(l) {
            if let Ok(v) = m[1].parse::<f64>() {
                out.insert("level".into(), v / 100.0);
            }
        }
    }
    out
}

fn z(p: f64) -> f64 {
    Normal::new(0.0, 1.0).unwrap().inverse_cdf(p)
}

fn phi(x: f64) -> f64 {
    Normal::new(0.0, 1.0).unwrap().cdf(x)
}

fn ans(title: &str, input: String, res_title: &str, res: String, rows: Vec<(&str, String)>, formula: &str) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", res_title, text(res)))];
    if !rows.is_empty() {
        pods.push(pod("Details", "Details", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())));
    }
    if !formula.is_empty() {
        pods.push(pod("Formula", "Formula", text(formula)));
    }
    Answer::new(title, pods)
}

fn tails(l: &str) -> (&'static str, &'static str) {
    if re!(r"\b(?:greater|right[- ]tailed|upper[- ]tailed|>)").is_match(l) {
        ("greater", "H₁: parameter > null value (right-tailed)")
    } else if re!(r"\b(?:less|left[- ]tailed|lower[- ]tailed|<)").is_match(l) {
        ("less", "H₁: parameter < null value (left-tailed)")
    } else {
        ("two", "H₁: parameter ≠ null value (two-tailed)")
    }
}

fn pvalue(stat: f64, cdf: &dyn Fn(f64) -> f64, tail: &str) -> f64 {
    match tail {
        "greater" => 1.0 - cdf(stat),
        "less" => cdf(stat),
        _ => 2.0 * (1.0 - cdf(stat.abs())),
    }
}

fn decision(p: f64, alpha: f64) -> String {
    if p < alpha {
        format!("reject H₀ at α = {} (p = {})", fmt(alpha), fmt(p))
    } else {
        format!("do not reject H₀ at α = {} (p = {})", fmt(alpha), fmt(p))
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let is_test = re!(r"\b(?:t-?test|z-?test|hypothesis|test for|test of|significance test)\b").is_match(&l);
    let is_ci = re!(r"\b(?:confidence interval|ci|interval)\b").is_match(&l);
    let is_size = re!(r"\bsample size\b").is_match(&l) && l.contains("margin");
    if !is_test && !is_ci && !is_size {
        return None;
    }
    let p = params(&l);
    let alpha = p.get("alpha").copied().unwrap_or(0.05);
    let level = p.get("level").copied().unwrap_or(1.0 - alpha);
    let (tail, h1) = tails(&l);
    let get = |k: &str| p.get(k).copied();
    let proportion = re!(r"binomial|proportion|p-?hat|p̂|\bp0\b").is_match(&l);
    let variance = re!(r"variance|standard deviation\b").is_match(&l) && !l.contains("t-test") && !l.contains("mean");
    let two_sample = re!(r"two[- ](?:sample|proportion|mean)|difference|between two").is_match(&l);

    // sample size for a margin of error
    if is_size {
        let e = get("e")?;
        let zc = z(1.0 - (1.0 - level) / 2.0);
        if let Some(sig) = get("sigma").or(get("s")) {
            let n = (zc * sig / e).powi(2);
            return Some(ans("sample size", format!("sample size for estimating a mean, σ = {}, margin of error {}, {}% confidence", fmt(sig), fmt(e), fmt(level * 100.0)), "Minimum sample size", format!("{}", n.ceil()), vec![("exact value", fmt(n)), ("critical value z", fmt(zc))], "n = (z σ / E)²"));
        }
        let ph = get("phat").unwrap_or(0.5);
        let n = zc * zc * ph * (1.0 - ph) / (e * e);
        return Some(ans("sample size", format!("sample size for estimating a proportion, p̂ = {}, margin of error {}, {}% confidence", fmt(ph), fmt(e), fmt(level * 100.0)), "Minimum sample size", format!("{}", n.ceil()), vec![("exact value", fmt(n)), ("critical value z", fmt(zc))], "n = z² p̂(1 − p̂) / E² (p̂ = 0.5 when unknown)"));
    }

    // tests
    if is_test {
        if proportion && !two_sample {
            if let (Some(p0), Some(n)) = (get("p0"), get("n")) {
                let ph = get("phat").or_else(|| get("x").map(|x| x / n))?;
                let se = (p0 * (1.0 - p0) / n).sqrt();
                let zs = (ph - p0) / se;
                let pv = pvalue(zs, &phi, tail);
                return Some(ans("proportion test", format!("one-proportion z-test, p₀ = {}, p̂ = {}, n = {}", fmt(p0), fmt(ph), fmt(n)), "Test statistic", format!("z = {}", fmt(zs)), vec![("p-value", fmt(pv)), ("hypotheses", format!("H₀: p = {}; {h1}", fmt(p0))), ("decision", decision(pv, alpha)), ("standard error", fmt(se))], "z = (p̂ − p₀) / √(p₀(1 − p₀)/n)"));
            }
        }
        if proportion && two_sample {
            if let (Some(p1), Some(p2), Some(n1), Some(n2)) = (get("phat"), get("phat2"), get("n"), get("n2")) {
                let pool = (p1 * n1 + p2 * n2) / (n1 + n2);
                let se = (pool * (1.0 - pool) * (1.0 / n1 + 1.0 / n2)).sqrt();
                let zs = (p1 - p2) / se;
                let pv = pvalue(zs, &phi, tail);
                return Some(ans("two-proportion test", format!("two-proportion z-test, p̂₁ = {}, n₁ = {}, p̂₂ = {}, n₂ = {}", fmt(p1), fmt(n1), fmt(p2), fmt(n2)), "Test statistic", format!("z = {}", fmt(zs)), vec![("p-value", fmt(pv)), ("pooled proportion", fmt(pool)), ("decision", decision(pv, alpha))], "z = (p̂₁ − p̂₂) / √(p̂(1 − p̂)(1/n₁ + 1/n₂)), p̂ = (x₁ + x₂)/(n₁ + n₂)"));
            }
            return Some(ans("two-proportion test", "two-proportion z-test".into(), "Test statistic", "z = (p̂₁ − p̂₂) / √(p̂(1 − p̂)(1/n₁ + 1/n₂))".into(), vec![("pooled proportion", "p̂ = (x₁ + x₂)/(n₁ + n₂)".into()), ("null hypothesis", "H₀: p₁ = p₂".into()), ("conditions", "n p̂ ≥ 10 and n(1 − p̂) ≥ 10 in each sample; independent random samples".into()), ("example", "two proportion test p1=0.42, n1=200, p2=0.35, n2=180".into())], ""));
        }
        if two_sample {
            if let (Some(x1), Some(x2), Some(s1), Some(s2), Some(n1), Some(n2)) = (get("xbar"), get("xbar2"), get("s"), get("s2"), get("n"), get("n2")) {
                let (v1, v2) = (s1 * s1 / n1, s2 * s2 / n2);
                let se = (v1 + v2).sqrt();
                let t = (x1 - x2) / se;
                let df = (v1 + v2).powi(2) / (v1 * v1 / (n1 - 1.0) + v2 * v2 / (n2 - 1.0));
                let dist = StudentsT::new(0.0, 1.0, df).ok()?;
                let pv = pvalue(t, &|x| dist.cdf(x), tail);
                return Some(ans("two-sample t-test", format!("Welch two-sample t-test, x̄₁ = {}, s₁ = {}, n₁ = {}, x̄₂ = {}, s₂ = {}, n₂ = {}", fmt(x1), fmt(s1), fmt(n1), fmt(x2), fmt(s2), fmt(n2)), "Test statistic", format!("t = {}", fmt(t)), vec![("degrees of freedom (Welch)", fmt(df)), ("p-value", fmt(pv)), ("decision", decision(pv, alpha))], "t = (x̄₁ − x̄₂) / √(s₁²/n₁ + s₂²/n₂)"));
            }
            return Some(ans("two-sample t-test", "hypothesis test for the difference between two means".into(), "Test statistic", "t = (x̄₁ − x̄₂ − Δ₀) / √(s₁²/n₁ + s₂²/n₂)".into(), vec![("null hypothesis", "H₀: μ₁ − μ₂ = Δ₀ (usually 0)".into()), ("degrees of freedom (Welch)", "(s₁²/n₁ + s₂²/n₂)² / [(s₁²/n₁)²/(n₁ − 1) + (s₂²/n₂)²/(n₂ − 1)]".into()), ("pooled variant", "s_p² = ((n₁ − 1)s₁² + (n₂ − 1)s₂²)/(n₁ + n₂ − 2), df = n₁ + n₂ − 2".into()), ("example", "t-test xbar1=5.2, s1=1.1, n1=30, xbar2=4.7, s2=1.3, n2=35 difference".into())], ""));
        }
        if let (Some(mu0), Some(xb), Some(n)) = (get("mu0"), get("xbar"), get("n")) {
            if let Some(sig) = get("sigma").filter(|_| !l.contains("t-test")) {
                let zs = (xb - mu0) / (sig / n.sqrt());
                let pv = pvalue(zs, &phi, tail);
                return Some(ans("z-test", format!("one-sample z-test, μ₀ = {}, x̄ = {}, σ = {}, n = {}", fmt(mu0), fmt(xb), fmt(sig), fmt(n)), "Test statistic", format!("z = {}", fmt(zs)), vec![("p-value", fmt(pv)), ("hypotheses", format!("H₀: μ = {}; {h1}", fmt(mu0))), ("decision", decision(pv, alpha))], "z = (x̄ − μ₀) / (σ/√n)"));
            }
            let s = get("s").or(get("sigma"))?;
            let t = (xb - mu0) / (s / n.sqrt());
            let df = n - 1.0;
            let dist = StudentsT::new(0.0, 1.0, df).ok()?;
            let pv = pvalue(t, &|x| dist.cdf(x), tail);
            let crit = dist.inverse_cdf(if tail == "two" { 1.0 - alpha / 2.0 } else { 1.0 - alpha });
            return Some(ans("t-test", format!("one-sample t-test, μ₀ = {}, x̄ = {}, s = {}, n = {}", fmt(mu0), fmt(xb), fmt(s), fmt(n)), "Test statistic", format!("t = {}", fmt(t)), vec![("degrees of freedom", fmt(df)), ("p-value", fmt(pv)), ("critical value", format!("±{}", fmt(crit))), ("hypotheses", format!("H₀: μ = {}; {h1}", fmt(mu0))), ("decision", decision(pv, alpha))], "t = (x̄ − μ₀) / (s/√n), with n − 1 degrees of freedom"));
        }
        return None;
    }

    // confidence intervals
    let zc = z(1.0 - (1.0 - level) / 2.0);
    let pct = fmt(level * 100.0);
    if l.contains("jeffreys") {
        if let (Some(n), Some(ph)) = (get("n"), get("phat").or_else(|| get("x").and_then(|x| get("n").map(|n| x / n)))) {
            let x = ph * n;
            let b = |q: f64| Beta::new(x + 0.5, n - x + 0.5).ok().map(|d| d.inverse_cdf(q));
            let (lo, hi) = (b((1.0 - level) / 2.0)?, b(1.0 - (1.0 - level) / 2.0)?);
            return Some(ans("Jeffreys interval", format!("{pct}% Jeffreys interval, n = {}, p̂ = {}", fmt(n), fmt(ph)), "Interval", format!("({}, {})", fmt(lo), fmt(hi)), vec![], "quantiles α/2 and 1 − α/2 of Beta(x + 1/2, n − x + 1/2)"));
        }
        return Some(ans("Jeffreys interval", "Jeffreys interval for a binomial proportion".into(), "Formula", "(B(α/2; x + 1/2, n − x + 1/2), B(1 − α/2; x + 1/2, n − x + 1/2))".into(), vec![("B(q; a, b)", "the q-quantile of the beta distribution".into()), ("origin", "the equal-tailed Bayesian credible interval under the Jeffreys prior Beta(1/2, 1/2)".into()), ("convention", "the lower limit is 0 when x = 0 and the upper limit is 1 when x = n".into())], ""));
    }
    if variance || l.contains("for variance") || l.contains("for a variance") || l.contains("standard deviation") && !l.contains("mean") {
        let want_sd = l.contains("standard deviation") && !l.contains("variance");
        let var = get("var").or_else(|| get("s").map(|s| s * s));
        if let (Some(v), Some(n)) = (var, get("n")) {
            let d = ChiSquared::new(n - 1.0).ok()?;
            let (lo, hi) = ((n - 1.0) * v / d.inverse_cdf(1.0 - (1.0 - level) / 2.0), (n - 1.0) * v / d.inverse_cdf((1.0 - level) / 2.0));
            let (lo, hi, what) = if want_sd { (lo.sqrt(), hi.sqrt(), "standard deviation") } else { (lo, hi, "variance") };
            return Some(ans("confidence interval", format!("{pct}% confidence interval for a {what}, s² = {}, n = {}", fmt(v), fmt(n)), "Interval", format!("({}, {})", fmt(lo), fmt(hi)), vec![("degrees of freedom", fmt(n - 1.0))], "((n − 1)s²/χ²_{1−α/2}, (n − 1)s²/χ²_{α/2})"));
        }
        let what = if want_sd { "standard deviation" } else { "variance" };
        return Some(ans("confidence interval", format!("confidence interval for a {what}"), "Formula", if want_sd { "(√((n − 1)s²/χ²_{1−α/2, n−1}), √((n − 1)s²/χ²_{α/2, n−1}))".into() } else { "((n − 1)s²/χ²_{1−α/2, n−1}, (n − 1)s²/χ²_{α/2, n−1})".into() }, vec![("assumption", "the population is normally distributed".into()), ("example", "CI for variance, s^2=21.4, n=300".into())], ""));
    }
    if proportion && two_sample {
        if let (Some(p1), Some(p2), Some(n1), Some(n2)) = (get("phat"), get("phat2"), get("n"), get("n2")) {
            let se = (p1 * (1.0 - p1) / n1 + p2 * (1.0 - p2) / n2).sqrt();
            let d = p1 - p2;
            return Some(ans("confidence interval", format!("{pct}% confidence interval for p₁ − p₂, p̂₁ = {}, n₁ = {}, p̂₂ = {}, n₂ = {}", fmt(p1), fmt(n1), fmt(p2), fmt(n2)), "Interval", format!("({}, {})", fmt(d - zc * se), fmt(d + zc * se)), vec![("point estimate", fmt(d)), ("margin of error", fmt(zc * se))], "(p̂₁ − p̂₂) ± z √(p̂₁(1 − p̂₁)/n₁ + p̂₂(1 − p̂₂)/n₂)"));
        }
        return Some(ans("confidence interval", "confidence interval for a difference of proportions".into(), "Formula", "(p̂₁ − p̂₂) ± z_{α/2} √(p̂₁(1 − p̂₁)/n₁ + p̂₂(1 − p̂₂)/n₂)".into(), vec![("conditions", "at least 10 successes and 10 failures in each sample".into())], ""));
    }
    if proportion {
        if let (Some(n), Some(ph)) = (get("n"), get("phat").or_else(|| get("x").and_then(|x| get("n").map(|n| x / n)))) {
            let se = (ph * (1.0 - ph) / n).sqrt();
            let w = {
                let d = 1.0 + zc * zc / n;
                let c = (ph + zc * zc / (2.0 * n)) / d;
                let h = zc * (ph * (1.0 - ph) / n + zc * zc / (4.0 * n * n)).sqrt() / d;
                (c - h, c + h)
            };
            return Some(ans("confidence interval", format!("{pct}% confidence interval for a binomial proportion, n = {}, p̂ = {}", fmt(n), fmt(ph)), "Wald interval", format!("({}, {})", fmt(ph - zc * se), fmt(ph + zc * se)), vec![("margin of error", fmt(zc * se)), ("Wilson score interval", format!("({}, {})", fmt(w.0), fmt(w.1))), ("critical value z", fmt(zc))], "p̂ ± z √(p̂(1 − p̂)/n)"));
        }
        return None;
    }
    if two_sample && l.contains("mean") {
        if let (Some(x1), Some(x2), Some(s1), Some(s2), Some(n1), Some(n2)) = (get("xbar"), get("xbar2"), get("s"), get("s2"), get("n"), get("n2")) {
            let (v1, v2) = (s1 * s1 / n1, s2 * s2 / n2);
            let df = (v1 + v2).powi(2) / (v1 * v1 / (n1 - 1.0) + v2 * v2 / (n2 - 1.0));
            let tc = StudentsT::new(0.0, 1.0, df).ok()?.inverse_cdf(1.0 - (1.0 - level) / 2.0);
            let d = x1 - x2;
            let me = tc * (v1 + v2).sqrt();
            return Some(ans("confidence interval", format!("{pct}% confidence interval for μ₁ − μ₂"), "Interval", format!("({}, {})", fmt(d - me), fmt(d + me)), vec![("point estimate", fmt(d)), ("margin of error", fmt(me)), ("degrees of freedom (Welch)", fmt(df))], "(x̄₁ − x̄₂) ± t √(s₁²/n₁ + s₂²/n₂)"));
        }
        return Some(ans("confidence interval", "confidence interval for a difference of means".into(), "Formula", "(x̄₁ − x̄₂) ± t_{α/2, df} √(s₁²/n₁ + s₂²/n₂)".into(), vec![("df (Welch)", "(s₁²/n₁ + s₂²/n₂)² / [(s₁²/n₁)²/(n₁ − 1) + (s₂²/n₂)²/(n₂ − 1)]".into()), ("paired data", "d̄ ± t_{α/2, n−1} s_d/√n".into())], ""));
    }
    if l.contains("mean") || get("xbar").is_some() {
        if let (Some(xb), Some(n)) = (get("xbar"), get("n")) {
            if let Some(sig) = get("sigma") {
                let me = zc * sig / n.sqrt();
                return Some(ans("confidence interval", format!("{pct}% confidence interval for a mean, x̄ = {}, σ = {}, n = {}", fmt(xb), fmt(sig), fmt(n)), "Interval", format!("({}, {})", fmt(xb - me), fmt(xb + me)), vec![("margin of error", fmt(me)), ("critical value z", fmt(zc))], "x̄ ± z σ/√n"));
            }
            let s = get("s")?;
            let tc = StudentsT::new(0.0, 1.0, n - 1.0).ok()?.inverse_cdf(1.0 - (1.0 - level) / 2.0);
            let me = tc * s / n.sqrt();
            return Some(ans("confidence interval", format!("{pct}% confidence interval for a mean, x̄ = {}, s = {}, n = {}", fmt(xb), fmt(s), fmt(n)), "Interval", format!("({}, {})", fmt(xb - me), fmt(xb + me)), vec![("margin of error", fmt(me)), ("critical value t", fmt(tc)), ("degrees of freedom", fmt(n - 1.0))], "x̄ ± t s/√n"));
        }
        if p.is_empty() {
            return Some(ans("confidence interval", "confidence interval for a population mean".into(), "Formula", "x̄ ± t_{α/2, n−1} s/√n (σ unknown); x̄ ± z_{α/2} σ/√n (σ known)".into(), vec![("common z values", "90%: 1.645, 95%: 1.960, 99%: 2.576".into()), ("example", "95% confidence interval xbar=52, s=8, n=40".into())], ""));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    fn short(s: &str) -> String {
        let r = crate::engine::query(s);
        serde_json::to_string(&r.pods).unwrap_or_default()
    }

    #[test]
    fn inference_answers() {
        assert!(short("t-test mu0=250, xbar=264, s=18, n=60").contains("t = 6.02"));
        assert!(short("test for binomial parameter p0=0.75, p-hat=0.84, n=40").contains("z = 1.31"));
        assert!(short("binomial confidence interval n=120, p-hat=0.42").contains("0.331"));
        assert!(short("CI for variance, s^2=21.4, n=300").contains("18.34"));
    }
}
