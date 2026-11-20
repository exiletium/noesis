//! Common drugs: their pharmacological classes, how they act and what they
//! are used for, plus the reverse lookup of drugs used for a condition.
//!
//! "drug categories for caffeine", "what class of drug is ibuprofen",
//! "drugs prescribed for type 2 diabetes".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | classes | mechanism | uses (conditions, ";"-separated)
const DRUGS: &[(&[&str], &str, &str, &str)] = &[
    (&["caffeine"], "central nervous system stimulant; methylxanthine", "blocks adenosine receptors", "drowsiness and fatigue; apnoea of prematurity; added to some pain relievers"),
    (&["ibuprofen", "advil", "motrin"], "nonsteroidal anti-inflammatory drug (NSAID); propionic acid derivative", "inhibits cyclooxygenase (COX-1 and COX-2), lowering prostaglandin production", "pain; fever; inflammation; arthritis"),
    (&["naproxen", "aleve"], "nonsteroidal anti-inflammatory drug (NSAID); propionic acid derivative", "inhibits cyclooxygenase (COX-1 and COX-2)", "pain; fever; inflammation; arthritis"),
    (&["aspirin", "acetylsalicylic acid"], "nonsteroidal anti-inflammatory drug (NSAID); salicylate; antiplatelet agent", "irreversibly inhibits cyclooxygenase, including platelet thromboxane production", "pain; fever; inflammation; prevention of heart attack and stroke"),
    (&["acetaminophen", "paracetamol", "tylenol"], "analgesic; antipyretic (not an anti-inflammatory)", "not fully understood; thought to inhibit prostaglandin synthesis in the central nervous system", "pain; fever"),
    (&["metformin"], "biguanide; oral antidiabetic", "reduces glucose production by the liver and improves insulin sensitivity", "type 2 diabetes"),
    (&["insulin"], "hormone; antidiabetic", "promotes glucose uptake by cells and storage as glycogen", "type 1 diabetes; type 2 diabetes"),
    (&["sitagliptin"], "dipeptidyl peptidase-4 (DPP-4) inhibitor; oral antidiabetic", "prolongs the action of incretin hormones, increasing insulin release after meals", "type 2 diabetes"),
    (&["glipizide"], "sulfonylurea; oral antidiabetic", "stimulates insulin release from pancreatic beta cells", "type 2 diabetes"),
    (&["empagliflozin"], "sodium-glucose cotransporter-2 (SGLT2) inhibitor", "makes the kidneys excrete glucose in the urine", "type 2 diabetes; heart failure; chronic kidney disease"),
    (&["semaglutide", "ozempic", "wegovy"], "glucagon-like peptide-1 (GLP-1) receptor agonist", "increases glucose-dependent insulin release, slows gastric emptying and reduces appetite", "type 2 diabetes; obesity"),
    (&["pioglitazone"], "thiazolidinedione; oral antidiabetic", "activates PPAR-γ, increasing insulin sensitivity", "type 2 diabetes"),
    (&["atorvastatin", "lipitor"], "statin (HMG-CoA reductase inhibitor)", "blocks cholesterol synthesis in the liver", "high cholesterol; prevention of cardiovascular disease"),
    (&["lisinopril"], "angiotensin-converting enzyme (ACE) inhibitor", "blocks conversion of angiotensin I to angiotensin II, relaxing blood vessels", "high blood pressure; heart failure; after heart attack"),
    (&["amlodipine"], "calcium channel blocker (dihydropyridine)", "blocks L-type calcium channels in vascular smooth muscle, relaxing arteries", "high blood pressure; angina"),
    (&["metoprolol"], "beta blocker (beta-1 selective)", "blocks beta-1 adrenergic receptors, slowing the heart and lowering its output", "high blood pressure; angina; heart failure; arrhythmias"),
    (&["omeprazole", "prilosec"], "proton pump inhibitor", "blocks the H+/K+ ATPase of stomach parietal cells, reducing acid", "gastro-oesophageal reflux; stomach ulcers"),
    (&["amoxicillin"], "penicillin antibiotic (beta-lactam)", "inhibits bacterial cell-wall synthesis", "bacterial infections"),
    (&["penicillin"], "beta-lactam antibiotic", "inhibits bacterial cell-wall synthesis", "bacterial infections"),
    (&["sertraline", "zoloft"], "selective serotonin reuptake inhibitor (SSRI); antidepressant", "blocks reuptake of serotonin", "depression; anxiety disorders; obsessive-compulsive disorder"),
    (&["fluoxetine", "prozac"], "selective serotonin reuptake inhibitor (SSRI); antidepressant", "blocks reuptake of serotonin", "depression; obsessive-compulsive disorder; bulimia"),
    (&["diazepam", "valium"], "benzodiazepine; anxiolytic; sedative", "enhances GABA-A receptor activity", "anxiety; muscle spasms; seizures; alcohol withdrawal"),
    (&["morphine"], "opioid analgesic", "activates mu-opioid receptors", "severe pain"),
    (&["nicotine"], "stimulant; nicotinic acetylcholine receptor agonist", "activates nicotinic acetylcholine receptors", "smoking cessation (as replacement therapy)"),
    (&["warfarin"], "anticoagulant; vitamin K antagonist", "blocks recycling of vitamin K, reducing clotting factors II, VII, IX and X", "prevention of blood clots; atrial fibrillation"),
    (&["levothyroxine"], "thyroid hormone (synthetic T4)", "replaces thyroxine", "hypothyroidism"),
    (&["prednisone"], "corticosteroid (glucocorticoid)", "binds glucocorticoid receptors, suppressing inflammation and immune responses", "inflammation; asthma; autoimmune diseases"),
    (&["diphenhydramine", "benadryl"], "first-generation H1 antihistamine; sedative", "blocks histamine H1 receptors (and crosses into the brain)", "allergies; insomnia; motion sickness"),
    (&["loratadine", "claritin"], "second-generation H1 antihistamine", "blocks peripheral histamine H1 receptors", "allergies"),
    (&["sildenafil", "viagra"], "phosphodiesterase-5 (PDE5) inhibitor", "raises cGMP, relaxing smooth muscle", "erectile dysfunction; pulmonary arterial hypertension"),
    (&["albuterol", "salbutamol"], "short-acting beta-2 agonist; bronchodilator", "activates beta-2 receptors, relaxing airway smooth muscle", "asthma; chronic obstructive pulmonary disease"),
];

/// Well-known interactions: (names, &[(interacting drug or group, effect)]).
const INTERACTIONS: &[(&[&str], &[(&str, &str)])] = &[
    (&["ibuprofen", "advil", "motrin"], &[
        ("warfarin and other anticoagulants", "higher risk of bleeding"),
        ("aspirin (low-dose)", "ibuprofen taken first can block aspirin's antiplatelet effect; more stomach bleeding"),
        ("other NSAIDs (naproxen, diclofenac)", "more gastrointestinal ulcers and bleeding, no added benefit"),
        ("SSRIs and SNRIs (sertraline, fluoxetine, venlafaxine)", "higher risk of bleeding"),
        ("corticosteroids (prednisone)", "more gastrointestinal ulcers and bleeding"),
        ("ACE inhibitors, angiotensin receptor blockers and diuretics", "weaker blood-pressure lowering; kidney injury, especially all three together"),
        ("lithium", "higher lithium levels (toxicity)"),
        ("methotrexate", "higher methotrexate levels (toxicity)"),
        ("alcohol", "more stomach bleeding"),
    ]),
    (&["aspirin", "acetylsalicylic acid"], &[
        ("warfarin, heparin and direct oral anticoagulants", "higher risk of bleeding"),
        ("ibuprofen and other NSAIDs", "less antiplatelet effect (ibuprofen) and more stomach bleeding"),
        ("SSRIs and SNRIs", "higher risk of bleeding"),
        ("corticosteroids", "more gastrointestinal ulcers and bleeding"),
        ("methotrexate", "higher methotrexate levels (toxicity)"),
        ("alcohol", "more stomach bleeding"),
    ]),
    (&["acetaminophen", "paracetamol", "tylenol"], &[
        ("alcohol (regular heavy use)", "higher risk of liver damage"),
        ("other acetaminophen-containing products (cold and flu remedies)", "accidental overdose"),
        ("warfarin", "raised INR with regular daily use"),
        ("carbamazepine, phenytoin, rifampin and isoniazid", "higher risk of liver toxicity"),
    ]),
    (&["warfarin", "coumadin"], &[
        ("aspirin, NSAIDs and antiplatelet drugs", "higher risk of bleeding"),
        ("antibiotics (metronidazole, ciprofloxacin, trimethoprim–sulfamethoxazole)", "raised INR, bleeding"),
        ("fluconazole and amiodarone", "raised INR, bleeding"),
        ("rifampin, carbamazepine and St John's wort", "lower INR, less protection from clots"),
        ("vitamin K-rich foods (leafy greens), in changing amounts", "unstable INR"),
    ]),
];

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    // "ibuprofen drug interactions"
    if let Some(m) = re!(r"^(?:(?:drug )?interactions (?:of|for|with) (.+)|(.+?) (?:drug )?interactions)$").captures(&l) {
        let name = m.get(1).or(m.get(2))?.as_str().trim();
        let (names, list) = INTERACTIONS.iter().find(|d| d.0.contains(&name))?;
        return Some(Answer::new(format!("{} drug interactions", names[0]), vec![
            pod("Input", "Input interpretation", text(format!("{} | drug interactions", names[0]))),
            primary(pod("Result", "Notable interactions", text(list.iter().map(|x| x.0).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Effects", table(Some(vec!["with", "effect"]), list.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())),
            pod("Note", "Note", text("The best-known interactions only, not a complete list and not medical advice.")),
        ]));
    }
    // reverse: "drugs prescribed for type 2 diabetes"
    if let Some(m) = re!(r"^(?:what |which )?(?:are )?(?:the )?(?:common )?(?:drugs|medications|medicines)(?: (?:that are |which are )?(?:prescribed|used|given|taken))? (?:for|to treat|against) (.+)$").captures(&l) {
        let want = m[1].trim().trim_start_matches("the ");
        let want = match want {
            "hypertension" | "high bp" => "high blood pressure",
            "hypercholesterolemia" | "hyperlipidemia" | "high cholesterol levels" => "high cholesterol",
            "gerd" | "acid reflux" | "heartburn" | "reflux" => "gastro-oesophageal reflux",
            "diabetes type 2" | "type ii diabetes" | "diabetes mellitus type 2" => "type 2 diabetes",
            "diabetes type 1" | "type i diabetes" => "type 1 diabetes",
            "headache" | "headaches" => "pain",
            w => w,
        };
        let hits: Vec<_> = DRUGS.iter().filter(|d| d.3.split("; ").any(|u| u == want)).collect();
        if hits.is_empty() {
            return None;
        }
        return Some(Answer::new(format!("drugs for {want}"), vec![
            pod("Input", "Input interpretation", text(format!("drugs used for {want}"))),
            primary(pod("Result", "Drugs", text(hits.iter().map(|d| d.0[0]).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Classes", table(Some(vec!["drug", "class", "how it works"]), hits.iter().map(|d| vec![text(d.0[0]), text(d.1), text(d.2)]).collect())),
            pod("Note", "Note", text("A short reference list, not medical advice; treatment is chosen by a clinician.")),
        ]));
    }
    let m = re!(r"^(?:what (?:class|type|kind|category) of (?:drug|medication|medicine) is |(?:drug|pharmacological|pharmacologic|therapeutic) (?:categories|category|classes|class|types?) (?:for|of) |what (?:drug )?class (?:is|does) )(.+?)(?: (?:in|belong to))?$|^(.+?) (?:drug|pharmacological) (?:class|classes|category|categories|type)$").captures(&l)?;
    let name = m.get(1).or(m.get(2))?.as_str().trim();
    let d = DRUGS.iter().find(|d| d.0.contains(&name))?;
    Some(Answer::new(format!("{} drug class", d.0[0]), vec![
        pod("Input", "Input interpretation", text(format!("{} | drug classes", d.0[0]))),
        primary(pod("Result", "Drug classes", text(d.1))),
        pod("Details", "Details", table(None, vec![
            vec![text("mechanism"), text(cap(d.2))],
            vec![text("used for"), text(d.3.replace("; ", ", "))],
        ])),
    ]))
}
