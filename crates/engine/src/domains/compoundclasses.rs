//! Named classes of chemical compounds and their common members.
//!
//! "mineral acids", "nucleosides", "steroids", "amino acids",
//! "nucleobases", "strong acids", "ionic diatomics".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names (';'-separated) | definition | members as "name:formula" separated by '|'
const CLASSES: &[(&str, &str, &str)] = &[
    ("mineral acids;inorganic acids;mineral acid", "acids derived from inorganic compounds; most are strong and fully dissociate in water",
     "hydrochloric acid:HCl|nitric acid:HNO₃|sulfuric acid:H₂SO₄|phosphoric acid:H₃PO₄|hydrofluoric acid:HF|hydrobromic acid:HBr|hydroiodic acid:HI|perchloric acid:HClO₄|boric acid:H₃BO₃|carbonic acid:H₂CO₃"),
    ("strong acids;common strong acids", "acids that dissociate completely in water",
     "hydrochloric acid:HCl|hydrobromic acid:HBr|hydroiodic acid:HI|nitric acid:HNO₃|sulfuric acid:H₂SO₄|perchloric acid:HClO₄|chloric acid:HClO₃"),
    ("strong bases;common strong bases", "bases that dissociate completely in water, mostly alkali and heavy alkaline earth hydroxides",
     "lithium hydroxide:LiOH|sodium hydroxide:NaOH|potassium hydroxide:KOH|rubidium hydroxide:RbOH|cesium hydroxide:CsOH|calcium hydroxide:Ca(OH)₂|strontium hydroxide:Sr(OH)₂|barium hydroxide:Ba(OH)₂"),
    ("nucleosides;nucleoside", "a nucleobase joined to a ribose or deoxyribose sugar (without phosphate)",
     "adenosine:C₁₀H₁₃N₅O₄|guanosine:C₁₀H₁₃N₅O₅|cytidine:C₉H₁₃N₃O₅|uridine:C₉H₁₂N₂O₆|thymidine (deoxythymidine):C₁₀H₁₄N₂O₅|inosine:C₁₀H₁₂N₄O₅|deoxyadenosine:C₁₀H₁₃N₅O₃|deoxyguanosine:C₁₀H₁₃N₅O₄|deoxycytidine:C₉H₁₃N₃O₄"),
    ("nucleobases;nucleotide bases;dna bases;bases in dna", "nitrogenous bases of nucleic acids: purines (adenine, guanine) and pyrimidines (cytosine, thymine, uracil)",
     "adenine (purine):C₅H₅N₅|guanine (purine):C₅H₅N₅O|cytosine (pyrimidine):C₄H₅N₃O|thymine (pyrimidine, DNA only):C₅H₆N₂O₂|uracil (pyrimidine, RNA only):C₄H₄N₂O₂"),
    ("nucleotides;nucleotide", "nucleosides with one or more phosphate groups; the monomers of DNA and RNA",
     "adenosine monophosphate (AMP):C₁₀H₁₄N₅O₇P|guanosine monophosphate (GMP):C₁₀H₁₄N₅O₈P|cytidine monophosphate (CMP):C₉H₁₄N₃O₈P|uridine monophosphate (UMP):C₉H₁₃N₂O₉P|thymidine monophosphate (TMP):C₁₀H₁₅N₂O₈P|adenosine triphosphate (ATP):C₁₀H₁₆N₅O₁₃P₃"),
    ("steroids;steroid;steroid hormones", "lipids with four fused rings (three six-membered and one five-membered), the sterane skeleton",
     "cholesterol:C₂₇H₄₆O|testosterone:C₁₉H₂₈O₂|estradiol:C₁₈H₂₄O₂|progesterone:C₂₁H₃₀O₂|cortisol:C₂₁H₃₀O₅|aldosterone:C₂₁H₂₈O₅|cholic acid:C₂₄H₄₀O₅|ergosterol:C₂₈H₄₄O|dexamethasone:C₂₂H₂₉FO₅|prednisone:C₂₁H₂₆O₅"),
    ("amino acids;standard amino acids;proteinogenic amino acids;the 20 amino acids;20 amino acids", "the 20 amino acids encoded by the standard genetic code (essential ones marked *)",
     "alanine (Ala, A):C₃H₇NO₂|arginine (Arg, R):C₆H₁₄N₄O₂|asparagine (Asn, N):C₄H₈N₂O₃|aspartic acid (Asp, D):C₄H₇NO₄|cysteine (Cys, C):C₃H₇NO₂S|glutamic acid (Glu, E):C₅H₉NO₄|glutamine (Gln, Q):C₅H₁₀N₂O₃|glycine (Gly, G):C₂H₅NO₂|histidine* (His, H):C₆H₉N₃O₂|isoleucine* (Ile, I):C₆H₁₃NO₂|leucine* (Leu, L):C₆H₁₃NO₂|lysine* (Lys, K):C₆H₁₄N₂O₂|methionine* (Met, M):C₅H₁₁NO₂S|phenylalanine* (Phe, F):C₉H₁₁NO₂|proline (Pro, P):C₅H₉NO₂|serine (Ser, S):C₃H₇NO₃|threonine* (Thr, T):C₄H₉NO₃|tryptophan* (Trp, W):C₁₁H₁₂N₂O₂|tyrosine (Tyr, Y):C₉H₁₁NO₃|valine* (Val, V):C₅H₁₁NO₂"),
    ("essential amino acids", "amino acids humans cannot synthesize and must obtain from food",
     "histidine:C₆H₉N₃O₂|isoleucine:C₆H₁₃NO₂|leucine:C₆H₁₃NO₂|lysine:C₆H₁₄N₂O₂|methionine:C₅H₁₁NO₂S|phenylalanine:C₉H₁₁NO₂|threonine:C₄H₉NO₃|tryptophan:C₁₁H₁₂N₂O₂|valine:C₅H₁₁NO₂"),
    ("ionic diatomics;ionic diatomic molecules;diatomic ionic compounds", "two-atom molecules held by a mainly ionic bond, typically gas-phase alkali halides",
     "sodium chloride:NaCl|potassium chloride:KCl|lithium fluoride:LiF|sodium fluoride:NaF|potassium bromide:KBr|cesium iodide:CsI|lithium hydride:LiH|potassium iodide:KI|rubidium chloride:RbCl|cesium fluoride:CsF"),
    ("diatomic elements;homonuclear diatomic molecules;diatomic molecules", "elements that exist as two-atom molecules under standard conditions",
     "hydrogen:H₂|nitrogen:N₂|oxygen:O₂|fluorine:F₂|chlorine:Cl₂|bromine:Br₂|iodine:I₂"),
    ("monosaccharides;simple sugars", "the simplest carbohydrates, which cannot be hydrolysed further",
     "glucose:C₆H₁₂O₆|fructose:C₆H₁₂O₆|galactose:C₆H₁₂O₆|ribose:C₅H₁₀O₅|deoxyribose:C₅H₁₀O₄|xylose:C₅H₁₀O₅|mannose:C₆H₁₂O₆"),
    ("disaccharides", "two monosaccharides joined by a glycosidic bond",
     "sucrose:C₁₂H₂₂O₁₁|lactose:C₁₂H₂₂O₁₁|maltose:C₁₂H₂₂O₁₁|trehalose:C₁₂H₂₂O₁₁|cellobiose:C₁₂H₂₂O₁₁"),
    ("neurotransmitters", "chemical messengers released at synapses",
     "acetylcholine:C₇H₁₆NO₂⁺|dopamine:C₈H₁₁NO₂|serotonin:C₁₀H₁₂N₂O|norepinephrine:C₈H₁₁NO₃|epinephrine:C₉H₁₃NO₃|GABA:C₄H₉NO₂|glutamate:C₅H₉NO₄|glycine:C₂H₅NO₂|histamine:C₅H₉N₃"),
    ("fatty acids;common fatty acids", "carboxylic acids with long aliphatic chains",
     "palmitic acid (16:0):C₁₆H₃₂O₂|stearic acid (18:0):C₁₈H₃₆O₂|oleic acid (18:1):C₁₈H₃₄O₂|linoleic acid (18:2):C₁₈H₃₂O₂|α-linolenic acid (18:3):C₁₈H₃₀O₂|arachidonic acid (20:4):C₂₀H₃₂O₂|lauric acid (12:0):C₁₂H₂₄O₂|myristic acid (14:0):C₁₄H₂₈O₂|EPA (20:5):C₂₀H₃₀O₂|DHA (22:6):C₂₂H₃₂O₂"),
    ("alkanes;straight-chain alkanes", "saturated hydrocarbons CₙH₂ₙ₊₂",
     "methane:CH₄|ethane:C₂H₆|propane:C₃H₈|butane:C₄H₁₀|pentane:C₅H₁₂|hexane:C₆H₁₄|heptane:C₇H₁₆|octane:C₈H₁₈|nonane:C₉H₂₀|decane:C₁₀H₂₂"),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"^(?:list (?:of |the )?|list all |all (?:the )?|what are (?:the )?|examples of |types of )").replace(l, "").to_string();
    let l = l.trim();
    let (names, def, members) = CLASSES.iter().find(|c| c.0.split(';').any(|n| n == l))?;
    let disp = names.split(';').next().unwrap();
    let list: Vec<(&str, &str)> = members.split('|').filter_map(|m| m.split_once(':')).collect();
    let rows: Vec<Vec<Content>> = list.iter().map(|(n, f)| vec![text(*n), text(*f)]).collect();
    Some(Answer::new(disp.to_string(), vec![
        pod("Input", "Input interpretation", text(format!("{disp} (class of compounds)"))),
        primary(pod("Result", "Members", text(list.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Common members", table(Some(vec!["compound", "formula"]), rows)),
        pod("Definition", "Definition", text(*def)),
    ]))
}
