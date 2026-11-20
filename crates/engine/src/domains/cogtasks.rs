//! Standard neuropsychological and cognitive tasks: what each measures, and
//! which tasks measure a given ability.
//!
//! "what does Iowa gambling task measure", "tasks to measure decision making",
//! "Stroop task", "letter naming".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | display | measures (abilities) | procedure
const TASKS: &[(&[&str], &str, &[&str], &str)] = &[
    (&["wais arithmetic subtest", "arithmetic subtest", "wechsler arithmetic subtest"], "WAIS Arithmetic subtest", &["arithmetic processing", "working memory", "numerical reasoning"], "solve orally presented arithmetic word problems mentally, within a time limit"),
    (&["mental arithmetic task", "serial subtraction task", "serial sevens", "serial 7s"], "mental arithmetic task", &["arithmetic processing", "working memory", "sustained attention"], "add or subtract mentally, for example counting backwards from 100 by sevens"),
    (&["number comparison task", "numerical comparison task", "magnitude comparison task"], "number comparison task", &["numerical magnitude processing", "arithmetic processing"], "decide quickly which of two numbers or dot arrays is larger; responses slow as the numbers get closer"),
    (&["benton facial recognition test", "benton face recognition test"], "Benton Facial Recognition Test", &["visual face recognition", "face perception"], "match a target face to photographs of the same person shown from other angles and under other lighting"),
    (&["cambridge face memory test", "cfmt"], "Cambridge Face Memory Test", &["visual face recognition", "face memory"], "learn six unfamiliar faces, then pick them out from distractors in new views and with added noise"),
    (&["visual patterns test", "visual pattern test"], "Visual Patterns Test", &["visual pattern recognition", "visual short-term memory"], "view a grid with some cells filled, then reproduce the pattern on a blank grid; the grids grow until errors appear"),
    (&["raven's progressive matrices", "ravens progressive matrices", "raven progressive matrices", "raven's matrices"], "Raven's Progressive Matrices", &["visual pattern recognition", "abstract reasoning", "fluid intelligence"], "choose the piece that completes a visual pattern in a matrix of figures"),
    (&["iowa gambling task", "iowa gambling test"], "Iowa gambling task", &["decision making", "decision making under uncertainty", "reward and punishment learning"], "choose cards from four decks, two with large gains but larger losses and two with small gains and smaller losses; healthy players learn to prefer the safe decks"),
    (&["cambridge gambling task"], "Cambridge gambling task", &["decision making", "risk taking", "impulsivity"], "bet a share of points on whether a token is hidden under a red or a blue box, with the odds shown"),
    (&["balloon analogue risk task", "balloon analog risk task"], "Balloon Analogue Risk Task", &["risk taking", "decision making"], "pump a virtual balloon for money per pump, cashing out before it bursts"),
    (&["delay discounting task", "delay discounting", "temporal discounting task", "intertemporal choice task"], "delay discounting task", &["impulsivity", "decision making", "self-control"], "choose between a smaller reward now and a larger reward later"),
    (&["game of dice task"], "Game of Dice Task", &["decision making", "decision making under risk"], "bet on the outcome of a die roll with explicit, fixed odds and payoffs"),
    (&["ultimatum game"], "ultimatum game", &["social decision making", "decision making", "fairness"], "a proposer splits a sum and the responder accepts or rejects; rejection leaves both with nothing"),
    (&["stroop task", "stroop test", "stroop color word test", "stroop colour word test"], "Stroop task", &["selective attention", "inhibitory control", "cognitive interference", "executive function"], "name the ink colour of colour words whose meaning conflicts with the ink"),
    (&["wisconsin card sorting test", "wisconsin card sorting task", "wcst"], "Wisconsin card sorting test", &["cognitive flexibility", "set shifting", "executive function", "abstract reasoning"], "sort cards by an unstated rule (colour, shape or number) that changes without warning"),
    (&["n-back task", "n back task", "n-back", "n-back test"], "n-back task", &["working memory"], "say whether each item matches the one n steps earlier"),
    (&["digit span task", "digit span", "digit span test"], "digit span task", &["short-term memory", "working memory"], "repeat sequences of digits forwards or backwards as they grow longer"),
    (&["tower of london", "tower of london task", "tower of london test"], "Tower of London", &["planning", "executive function", "problem solving"], "rearrange coloured balls on pegs to match a goal in the fewest moves"),
    (&["go/no-go task", "go no go task", "go/no go task", "go-no-go task"], "go/no-go task", &["response inhibition", "inhibitory control", "sustained attention"], "respond to frequent go stimuli and withhold the response to no-go stimuli"),
    (&["stop-signal task", "stop signal task"], "stop-signal task", &["response inhibition", "inhibitory control", "impulsivity"], "respond quickly, but cancel the response when a stop signal follows the cue"),
    (&["eriksen flanker task", "flanker task", "flanker test"], "Eriksen flanker task", &["selective attention", "inhibitory control", "conflict monitoring"], "report the direction of a central arrow flanked by congruent or incongruent arrows"),
    (&["trail making test", "trail making task"], "Trail Making Test", &["processing speed", "visual attention", "cognitive flexibility", "set shifting"], "connect numbered circles in order (part A), then alternate numbers and letters (part B)"),
    (&["continuous performance task", "continuous performance test"], "continuous performance task", &["sustained attention", "vigilance", "impulsivity"], "watch a long stream of stimuli and respond only to rare targets"),
    (&["mental rotation task", "mental rotation test"], "mental rotation task", &["spatial ability", "visuospatial processing"], "judge whether two drawings show the same object rotated or mirror images; time grows with the angle"),
    (&["posner cueing task", "posner cuing task", "posner task", "spatial cueing task"], "Posner cueing task", &["spatial attention", "attentional orienting"], "detect a target that appears where a cue pointed (valid) or elsewhere (invalid)"),
    (&["rapid automatized naming", "ran task", "letter naming", "letter naming task", "rapid naming task", "letter naming fluency"], "rapid automatized naming (letter naming)", &["naming speed", "processing speed", "reading fluency"], "name a grid of familiar letters, digits, colours or objects as fast as possible; speed predicts reading fluency"),
    (&["corsi block-tapping test", "corsi block tapping test", "corsi blocks", "corsi block test"], "Corsi block-tapping test", &["visuospatial working memory", "short-term memory", "working memory"], "tap a growing sequence of blocks in the order shown"),
    (&["verbal fluency test", "verbal fluency task", "phonemic fluency test", "category fluency test"], "verbal fluency test", &["executive function", "semantic memory", "language"], "name as many words as possible in a minute from a letter or a category"),
    (&["wechsler adult intelligence scale"], "Wechsler Adult Intelligence Scale", &["intelligence", "working memory", "processing speed", "verbal comprehension", "perceptual reasoning"], "a battery of subtests yielding index scores and a full-scale IQ"),
];

fn norm(s: &str) -> String {
    s.trim().trim_end_matches(['?', '.']).trim().replace(['–', '—'], "-").to_string()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = norm(&q.lower);
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    // "tasks to measure decision making", "which tests measure working memory"
    if let Some(m) = re!(r"^(?:what |which )?(?:are )?(?:the )?(?:cognitive |neuropsychological |psychological |behavio(?:u)?ral )?(?:tasks|tests|paradigms)(?: (?:to|that|which|for|used to))? (?:measure|measures|assess|assesses|test|tests|of|for) (.+)$").captures(&l) {
        let want = m[1].trim().trim_start_matches("the ").to_string();
        let hits: Vec<_> = TASKS.iter().filter(|t| t.2.iter().any(|a| *a == want || a.starts_with(&format!("{want} ")) || (want.len() > 5 && a.contains(want.as_str())))).collect();
        if hits.is_empty() {
            return None;
        }
        let rows = hits.iter().map(|t| vec![text(t.1), text(t.2.join(", ")), text(t.3)]).collect();
        return Some(Answer::new(format!("tasks measuring {want}"), vec![
            pod("Input", "Input interpretation", text(format!("cognitive tasks | measuring {want}"))),
            primary(pod("Result", "Tasks", text(hits.iter().map(|t| t.1).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Details", table(Some(vec!["task", "measures", "procedure"]), rows)),
        ]));
    }
    // a bare ability or list of abilities: "arithmetic processing", "visual pattern recognition and visual face recognition"
    {
        let parts: Vec<&str> = re!(r"\s*,\s*(?:and\s+)?|\s+and\s+").split(&l).map(|s| s.trim()).collect();
        if (parts.len() >= 2 || re!(r"processing$|recognition$").is_match(&l)) && parts.iter().all(|p| TASKS.iter().any(|t| t.2.contains(p))) && !TASKS.iter().any(|t| t.0.contains(&l.as_str())) {
            let hits: Vec<_> = TASKS.iter().filter(|t| parts.iter().any(|p| t.2.contains(p))).collect();
            let rows = hits.iter().map(|t| vec![text(t.1), text(t.2.join(", ")), text(t.3)]).collect();
            return Some(Answer::new(format!("tasks measuring {}", parts.join(" and ")), vec![
                pod("Input", "Input interpretation", text(format!("{} | cognitive ability", parts.join(" | ")))),
                primary(pod("Result", "Tasks that measure it", text(hits.iter().map(|t| t.1).collect::<Vec<_>>().join(" | ")))),
                pod("Table", "Details", table(Some(vec!["task", "measures", "procedure"]), rows)),
            ]));
        }
    }
    let name = re!(r"^(?:what (?:does|do) (?:the |an? )?(.+?) (?:measure|test|assess)|what is (?:the |an? )?(.+?) used (?:for|to measure)|(?:the )?(.+?)(?: measures| what it measures)?)$").captures(&l)?;
    let name = name.get(1).or(name.get(2)).or(name.get(3))?.as_str().trim();
    let t = TASKS.iter().find(|t| t.0.contains(&name))?;
    Some(Answer::new(t.1, vec![
        pod("Input", "Input interpretation", text(format!("{} | cognitive task", t.1))),
        primary(pod("Result", "Measures", text(t.2.join(", ")))),
        pod("Procedure", "Procedure", text(t.3)),
    ]))
}
