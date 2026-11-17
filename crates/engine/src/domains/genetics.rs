//! Molecular biology: DNA and RNA sequences (composition, GC content,
//! molecular weight, melting temperature, complements, transcription and
//! translation), the genetic code and codons, chromosome numbers and
//! genome sizes of model organisms, the human insulin sequence, and the
//! steps of core metabolic pathways.

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

/// Codon (RNA) → one-letter amino acid; '*' is a stop.
const CODE: &str = "UUUFUUCFUUALUUGLCUULCUCLCUALCUGLAUUIAUCIAUAIAUGMGUUVGUCVGUAVGUGVUCUSUCCSUCASUCGSCCUPCCCPCCAPCCGPACUTACCTACATACGTGCUAGCCAGCAAGCGAUAUYUACYUAA*UAG*CAUHCACHCAAQCAGQAAUNAACNAAAKAAGKGAUDGACDGAAEGAGEUGUCUGCCUGA*UGGWCGURCGCRCGARCGGRAGUSAGCSAGARAGGRGGUGGGCGGGAGGGGG";

fn codon_table() -> &'static Vec<(&'static str, char)> {
    static T: OnceLock<Vec<(&'static str, char)>> = OnceLock::new();
    T.get_or_init(|| (0..64).map(|i| (&CODE[i * 4..i * 4 + 3], CODE.as_bytes()[i * 4 + 3] as char)).collect())
}

fn amino(c: &str) -> char {
    codon_table().iter().find(|(k, _)| *k == c).map(|x| x.1).unwrap_or('?')
}

/// one letter, three letters, name
const AMINO: &[(char, &str, &str)] = &[
    ('A', "Ala", "alanine"), ('R', "Arg", "arginine"), ('N', "Asn", "asparagine"), ('D', "Asp", "aspartic acid"),
    ('C', "Cys", "cysteine"), ('Q', "Gln", "glutamine"), ('E', "Glu", "glutamic acid"), ('G', "Gly", "glycine"),
    ('H', "His", "histidine"), ('I', "Ile", "isoleucine"), ('L', "Leu", "leucine"), ('K', "Lys", "lysine"),
    ('M', "Met", "methionine"), ('F', "Phe", "phenylalanine"), ('P', "Pro", "proline"), ('S', "Ser", "serine"),
    ('T', "Thr", "threonine"), ('W', "Trp", "tryptophan"), ('Y', "Tyr", "tyrosine"), ('V', "Val", "valine"),
    ('*', "Stop", "stop"),
];

fn amino_name(c: char) -> &'static str {
    AMINO.iter().find(|a| a.0 == c).map(|a| a.2).unwrap_or("unknown")
}

fn complement(s: &str, rna: bool) -> String {
    s.chars().map(|c| match c {
        'A' => if rna { 'U' } else { 'T' },
        'T' | 'U' => 'A',
        'G' => 'C',
        'C' => 'G',
        x => x,
    }).collect()
}

fn translate(rna: &str) -> String {
    rna.as_bytes().chunks(3).filter(|c| c.len() == 3).map(|c| amino(std::str::from_utf8(c).unwrap_or(""))).collect()
}

/// Molecular weight of a single strand (g/mol): DNA without a 5′
/// phosphate, RNA with a 5′ triphosphate (common oligo calculator formulas).
fn mol_weight(s: &str, rna: bool) -> f64 {
    let cnt = |b: char| s.chars().filter(|c| *c == b).count() as f64;
    if rna {
        cnt('A') * 329.21 + cnt('U') * 306.17 + cnt('C') * 305.18 + cnt('G') * 345.21 + 159.0
    } else {
        cnt('A') * 313.21 + cnt('T') * 304.2 + cnt('C') * 289.18 + cnt('G') * 329.21 - 61.96
    }
}

/// Basic melting temperature: the Wallace rule below 14 nucleotides, the
/// Marmur–Doty GC formula above.
fn melting(s: &str) -> (f64, &'static str) {
    let len = s.len() as f64;
    let gc = s.chars().filter(|c| matches!(c, 'G' | 'C')).count() as f64;
    if s.len() < 14 {
        (2.0 * (len - gc) + 4.0 * gc, "Wallace rule: Tm = 2 °C × (A+T) + 4 °C × (G+C)")
    } else {
        (64.9 + 41.0 * (gc - 16.4) / len, "Tm = 64.9 °C + 41 °C × (G+C − 16.4)/N")
    }
}

/// A DNA or RNA sequence written in capitals.
fn sequence_of(raw: &str) -> Option<(String, bool)> {
    let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() < 6 || !s.chars().all(|c| matches!(c, 'A' | 'C' | 'G' | 'T' | 'U')) {
        return None;
    }
    let rna = s.contains('U');
    if rna && s.contains('T') {
        return None;
    }
    Some((s, rna))
}

fn sequence(q: &Query) -> Option<Answer> {
    let raw = q.raw.trim().trim_end_matches(['?', '.']);
    // "ATTGGGCAAAAA molecular weight", "melting point GCTCGTAGCTACGT", "reverse complement of ATGC"
    let (op, body) = if let Some(m) = re!(r"(?i)^(reverse complement|complement|transcription|transcribe|translate|translation|molecular weight|molecular mass|mass|melting point|melting temperature|tm|gc content|gc-content|length)\s+(?:of\s+)?(?:the\s+)?(?:sequence\s+)?([ACGTUacgtu ]+)$").captures(raw) {
        (Some(m[1].to_lowercase()), m[2].to_string())
    } else if let Some(m) = re!(r"(?i)^([ACGTUacgtu ]+?)\s+(reverse complement|complement|transcription|translation|molecular weight|molecular mass|mass|melting point|melting temperature|tm|gc content|gc-content|length)$").captures(raw) {
        (Some(m[2].to_lowercase()), m[1].to_string())
    } else {
        (None, raw.to_string())
    };
    // a bare sequence must be in capitals ("GATTACA"), an operation may be lower case
    // a sequence written in lower case ("mass of a cat") must be a single
    // word, and long unless the operation only makes sense for sequences
    let lowercase = body.chars().any(|c| c.is_ascii_lowercase());
    let seq_only = matches!(op.as_deref(), Some("reverse complement" | "complement" | "transcription" | "transcribe" | "translate" | "translation"));
    if lowercase && (body.trim().contains(' ') || (!seq_only && body.trim().len() < 8)) {
        return None;
    }
    let body = if op.is_some() { body.to_uppercase() } else { body };
    let (s, rna) = sequence_of(&body).or_else(|| if seq_only && body.trim().len() >= 2 { let t: String = body.split_whitespace().collect(); t.chars().all(|c| "ACGTU".contains(c)).then(|| (t.clone(), t.contains('U'))) } else { None })?;
    let len = s.len();
    let cnt = |b: char| s.chars().filter(|c| *c == b).count();
    let gc = cnt('G') + cnt('C');
    let gc_pct = 100.0 * gc as f64 / len as f64;
    let comp = complement(&s, rna);
    let revcomp: String = comp.chars().rev().collect();
    let mrna = if rna { s.clone() } else { s.replace('T', "U") };
    let protein = translate(&mrna);
    let mw = mol_weight(&s, rna);
    let (tm, tm_how) = melting(&s);
    let kind = if rna { "RNA" } else { "DNA" };
    let (title, main) = match op.as_deref() {
        Some("reverse complement") => ("Reverse complement", format!("5′-{revcomp}-3′")),
        Some("complement") => ("Complement", format!("3′-{comp}-5′")),
        Some("transcription" | "transcribe") => ("mRNA", format!("5′-{mrna}-3′")),
        Some("translate" | "translation") => ("Translation", protein.clone()),
        Some("molecular weight" | "molecular mass" | "mass") => ("Molecular weight", format!("{} g/mol (single strand)", n((mw * 100.0).round() / 100.0))),
        Some("melting point" | "melting temperature" | "tm") => ("Melting temperature", format!("{} °C", n((tm * 10.0).round() / 10.0))),
        Some("gc content" | "gc-content") => ("GC content", format!("{}%", n((gc_pct * 100.0).round() / 100.0))),
        Some("length") => ("Length", format!("{len} nucleotides")),
        _ => ("Sequence", format!("{len}-nucleotide {kind} sequence, {}% GC", n((gc_pct * 10.0).round() / 10.0))),
    };
    let counts = if rna { vec![('A', cnt('A')), ('C', cnt('C')), ('G', cnt('G')), ('U', cnt('U'))] } else { vec![('A', cnt('A')), ('C', cnt('C')), ('G', cnt('G')), ('T', cnt('T'))] };
    let comp_rows: Vec<Vec<Content>> = counts.iter().map(|(b, k)| vec![text(b.to_string()), text(k.to_string()), text(format!("{}%", n((100.0 * *k as f64 / len as f64 * 10.0).round() / 10.0)))]).collect();
    let props = vec![
        vec![text("length"), text(format!("{len} nt"))],
        vec![text("GC content"), text(format!("{}%", n((gc_pct * 100.0).round() / 100.0)))],
        vec![text("molecular weight"), text(format!("{} g/mol (single strand)", n((mw * 100.0).round() / 100.0)))],
        vec![text("melting temperature"), text(format!("{} °C ({tm_how})", n((tm * 10.0).round() / 10.0)))],
        vec![text("complement"), text(format!("3′-{comp}-5′"))],
        vec![text("reverse complement"), text(format!("5′-{revcomp}-3′"))],
        vec![text(if rna { "sequence" } else { "transcribed mRNA" }), text(format!("5′-{mrna}-3′"))],
        vec![text("translation (frame 1)"), text(if protein.is_empty() { "(too short)".to_string() } else { protein.clone() })],
    ];
    Some(Answer::new(format!("{kind} sequence"), vec![
        input(format!("{kind} sequence 5′-{s}-3′")),
        primary(pod("Result", title, text(main))),
        pod("Properties", "Sequence properties", table(None, props)),
        pod("Composition", "Base composition", table(Some(vec!["base", "count", "fraction"]), comp_rows)),
    ]))
}

fn codons(l: &str) -> Option<Answer> {
    if matches!(l, "genetic code" | "the genetic code" | "codon table" | "standard genetic code" | "dna codon table" | "rna codon table" | "codon chart") {
        let rows: Vec<Vec<Content>> = AMINO.iter().map(|(c, three, name)| {
            let cs: Vec<&str> = codon_table().iter().filter(|x| x.1 == *c).map(|x| x.0).collect();
            vec![text(name.to_string()), text(format!("{three} ({c})")), text(cs.join(", "))]
        }).collect();
        return Some(Answer::new("genetic code", vec![input("standard genetic code"), primary(pod("Result", "Codons by amino acid (mRNA)", table(Some(vec!["amino acid", "code", "codons"]), rows))), pod("Note", "Note", text("AUG (methionine) is the usual start codon; UAA, UAG and UGA are stop codons. 61 sense codons encode 20 amino acids."))]));
    }
    // "codon AUG", "what does GCU code for", "AUG codon"
    if let Some(m) = re!(r"^(?:codon\s+([acgtu]{3})|([acgtu]{3})\s+codon|what does ([acgtu]{3}) code for|what amino acid (?:does|is) ([acgtu]{3})(?: code for)?)$").captures(l) {
        let c = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4))?.as_str().to_uppercase().replace('T', "U");
        let a = amino(&c);
        let name = amino_name(a);
        let main = match (c.as_str(), a) {
            ("AUG", _) => "methionine (Met, M); also the start codon".to_string(),
            (_, '*') => "stop codon".to_string(),
            _ => format!("{name} ({}, {a})", AMINO.iter().find(|x| x.0 == a).map(|x| x.1).unwrap_or("")),
        };
        return Some(Answer::new("codon", vec![input(format!("codon {c}")), primary(pod("Result", "Amino acid", text(main))), pod("DNA", "DNA coding strand", text(c.replace('U', "T")))]));
    }
    // "alanine codons", "codons for leucine", "stop codons"
    let m = re!(r"^(?:codons? (?:for|of) ([a-z ]+)|([a-z ]+?) codons?)$").captures(l)?;
    let name = m.get(1).or(m.get(2))?.as_str().trim();
    let (c, three, full) = AMINO.iter().find(|(c, three, full)| *full == name || three.to_lowercase() == name || (name.len() == 1 && c.to_ascii_lowercase().to_string() == name) || (name == "start" && *c == 'M') || (name == "stop" && *c == '*'))?;
    let cs: Vec<&str> = codon_table().iter().filter(|x| x.1 == *c).map(|x| x.0).collect();
    Some(Answer::new("codons", vec![
        input(format!("codons for {full}")),
        primary(pod("Result", "mRNA codons", text(cs.join(", ")))),
        pod("DNA", "DNA codons (coding strand)", text(cs.iter().map(|x| x.replace('U', "T")).collect::<Vec<_>>().join(", "))),
        pod("Code", "Abbreviations", text(format!("{three}, {c}"))),
    ]))
}

/// organism names | diploid chromosome number (2n) | genome size | genes | note
const GENOMES: &str = "\
human,humans,homo sapiens,people|46|3.1 Gbp|about 20,000 protein-coding genes|22 pairs of autosomes and one pair of sex chromosomes (XX or XY)
chimpanzee,chimp,chimps,chimpanzees,pan troglodytes|48|3.1 Gbp|about 20,000|human chromosome 2 corresponds to two fused ape chromosomes
gorilla,gorillas|48|3.0 Gbp|about 20,000|
orangutan,orangutans|48|3.1 Gbp|about 20,000|
rhesus macaque,rhesus monkey,macaque|42|2.9 Gbp|about 21,000|
dog,dogs,canis familiaris|78|2.4 Gbp|about 19,000|
cat,cats,felis catus|38|2.5 Gbp|about 20,000|
horse,horses,equus caballus|64|2.5 Gbp|about 20,000|
donkey,donkeys|62|2.4 Gbp||
cow,cows,cattle,bos taurus|60|2.7 Gbp|about 22,000|
pig,pigs,sus scrofa|38|2.5 Gbp|about 21,000|
sheep|54|2.6 Gbp|about 20,000|
goat,goats|60|2.9 Gbp||
rabbit,rabbits|44|2.7 Gbp||
mouse,mice,house mouse,mus musculus|40|2.7 Gbp|about 22,000|the main mammalian model organism
rat,rats,rattus norvegicus|42|2.9 Gbp|about 22,000|
guinea pig|64|2.7 Gbp||
golden hamster,hamster|44|2.5 Gbp||
elephant,african elephant|56|3.4 Gbp||
bottlenose dolphin,dolphin|44|2.6 Gbp||
chicken,chickens,gallus gallus|78|1.1 Gbp|about 17,000|includes many tiny microchromosomes
zebrafish,danio rerio|50|1.4 Gbp|about 26,000|
african clawed frog,xenopus laevis|36|3.1 Gbp||allotetraploid
fruit fly,drosophila,drosophila melanogaster|8|143 Mbp|about 14,000|
honey bee,honeybee|32|250 Mbp|about 12,000|females are diploid (32); males (drones) are haploid (16)
roundworm,c. elegans,caenorhabditis elegans,nematode|12|100 Mbp|about 20,000|hermaphrodites have 2n = 12; males 11 (XO)
yeast,baker's yeast,saccharomyces cerevisiae,brewer's yeast|32|12.1 Mbp|about 6,000|16 chromosomes in the haploid cell
mouse-ear cress,mouseear cress,thale cress,arabidopsis,arabidopsis thaliana|10|135 Mbp|about 27,000|the main model plant; 5 chromosomes per haploid set
rice,oryza sativa|24|430 Mbp|about 37,000|
maize,corn,zea mays|20|2.3 Gbp|about 39,000|
wheat,bread wheat,common wheat|42|17 Gbp|about 107,000|hexaploid (6 sets of 7 chromosomes)
potato|48|840 Mbp|about 39,000|tetraploid
tomato|24|900 Mbp|about 35,000|
banana|33|520 Mbp||cultivated bananas are triploid and seedless
strawberry,garden strawberry|56|810 Mbp||octoploid
e. coli,escherichia coli|1|4.6 Mbp|about 4,300|a single circular chromosome
";

fn genomes(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:how many chromosomes (?:do|does|are in|in) (?:an? |the )?(.+?)(?: have)?|(?:number of )?chromosomes? (?:(?:do|does|are) )?(?:in |of |for )?(?:an? |the )?(.+?)(?: have)?|(.+?) chromosomes?(?: number)?|(?:the )?genome (?:of|size of) (?:an? |the )?(.+)|(.+?) genome(?: size)?)$").captures(l)?;
    let what = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4)).or(m.get(5))?.as_str();
    let items: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+|\s+and\s+)\s*").split(what).filter(|s| !s.trim().is_empty()).collect();
    let rows: Option<Vec<Vec<&str>>> = items.iter().map(|it| {
        let it = it.trim().trim_start_matches("a ").trim_start_matches("the ");
        GENOMES.lines().map(|line| line.split('|').collect::<Vec<_>>()).find(|p| p[0].split(',').any(|nm| nm == it))
    }).collect();
    let rows = rows?;
    let genome = l.contains("genome");
    let name = |p: &Vec<&str>| p[0].split(',').next().unwrap_or("").to_string();
    let main = if rows.len() == 1 {
        let p = &rows[0];
        if genome { format!("{} ({}; {} chromosomes)", p[2], if p[3].is_empty() { "genes not listed".to_string() } else if p[3].contains("genes") { p[3].to_string() } else { format!("{} genes", p[3]) }, p[1]) } else { format!("{} (diploid number, 2n)", p[1]) }
    } else {
        rows.iter().map(|p| format!("{}: {}", name(p), if genome { p[2].to_string() } else { p[1].to_string() })).collect::<Vec<_>>().join(" | ")
    };
    let t: Vec<Vec<Content>> = rows.iter().map(|p| vec![text(name(p)), text(p[1]), text(p[2]), text(p[3]), text(p[4])]).collect();
    Some(Answer::new("chromosomes", vec![
        input(format!("{} | {}", rows.iter().map(name).collect::<Vec<_>>().join(", "), if genome { "genome" } else { "chromosome number" })),
        primary(pod("Result", if genome { "Genome" } else { "Chromosomes" }, text(main))),
        pod("Table", "Genomes", table(Some(vec!["organism", "chromosomes (2n)", "genome size", "genes", "note"]), t)),
    ]))
}

/// organism | symbols and names | location | protein | length (aa) | function | homologs
const GENES: &str = "\
human|brca1,breast cancer 1|chromosome 17q21.31|breast cancer type 1 susceptibility protein|1863|DNA double-strand-break repair by homologous recombination; tumour suppressor. Inherited mutations raise breast and ovarian cancer risk|mouse Brca1 (chromosome 11), rat Brca1, dog BRCA1, chicken BRCA1; no fruit fly or yeast ortholog
human|brca2,breast cancer 2|chromosome 13q13.1|breast cancer type 2 susceptibility protein|3418|loads RAD51 onto DNA in homologous recombination; tumour suppressor|mouse Brca2 (chromosome 5), plant and fungal BRCA2-like proteins
human|tp53,p53|chromosome 17p13.1|cellular tumour antigen p53|393|transcription factor that halts the cell cycle or triggers apoptosis after DNA damage; the most often mutated gene in human cancer|mouse Trp53 (chromosome 11), paralogs TP63 and TP73, fruit fly p53
mouse|trp53,tp53,p53|chromosome 11|cellular tumour antigen p53|390|tumour suppressor; knockout mice develop tumours early|human TP53
human|ccr5,cd195|chromosome 3p21.31|C-C chemokine receptor type 5|352|chemokine receptor on T cells and macrophages; the main co-receptor HIV uses to enter cells. The CCR5-Δ32 deletion (about 10% allele frequency in northern Europe) gives homozygotes strong resistance to R5 HIV|mouse Ccr5 (chromosome 9), chimpanzee CCR5, rhesus macaque CCR5; paralogs CCR2, CCR1, CCR3
human|scnn1a,enac alpha|chromosome 12p13.31|amiloride-sensitive sodium channel subunit alpha (ENaC α)|669|epithelial sodium channel in kidney, lung and colon; controls sodium reabsorption and blood pressure. Loss-of-function mutations cause pseudohypoaldosteronism type 1|mouse Scnn1a (chromosome 6), rat Scnn1a, Xenopus scnn1a; paralogs SCNN1B, SCNN1G, SCNN1D
mouse|scnn1a,enac alpha|chromosome 6|amiloride-sensitive sodium channel subunit alpha (ENaC α)|699|epithelial sodium channel; knockout mice die soon after birth because their lungs cannot clear fluid|human SCNN1A, rat Scnn1a
human|msh2,muts homolog 2|chromosome 2p21–p16.3|DNA mismatch repair protein Msh2|934|recognizes mismatched bases (with MSH6 as MutSα, or MSH3 as MutSβ); inherited mutations cause Lynch syndrome (hereditary non-polyposis colorectal cancer)|E. coli mutS, yeast MSH2, mouse Msh2 (chromosome 17)
e. coli|muts|chromosome (about 2.78 Mb)|DNA mismatch repair protein MutS|853|binds mismatched base pairs after replication and recruits MutL and MutH for methyl-directed mismatch repair|human MSH2, MSH3 and MSH6; yeast MSH2
human|mlh1,mutl homolog 1|chromosome 3p22.2|DNA mismatch repair protein Mlh1|756|mismatch repair (with PMS2 as MutLα); Lynch syndrome gene|E. coli mutL, yeast MLH1
e. coli|mutl|chromosome|DNA mismatch repair protein MutL|615|links mismatch recognition by MutS to the MutH endonuclease|human MLH1 and PMS2
human|nampt,visfatin,pbef|chromosome 7q22.3|nicotinamide phosphoribosyltransferase|491|rate-limiting enzyme of the NAD⁺ salvage pathway|mouse Nampt (chromosome 12), bacterial nadV
mouse|nampt,visfatin,pbef|chromosome 12|nicotinamide phosphoribosyltransferase|491|rate-limiting enzyme of the NAD⁺ salvage pathway|human NAMPT
human|cftr|chromosome 7q31.2|cystic fibrosis transmembrane conductance regulator|1480|chloride channel in epithelia; mutations (most often ΔF508) cause cystic fibrosis|mouse Cftr (chromosome 6); an ABC transporter family member
human|hbb,beta globin|chromosome 11p15.4|haemoglobin subunit beta|147|oxygen transport in red blood cells; the Glu6Val mutation causes sickle-cell disease|mouse Hbb-bs and Hbb-bt; paralogs HBD, HBG1, HBG2, HBE1, myoglobin MB
human|ins,insulin|chromosome 11p15.5|insulin (preproinsulin)|110|hormone made by pancreatic β cells that lowers blood glucose|mouse Ins1 and Ins2, paralogs IGF1 and IGF2
human|apoe,apolipoprotein e|chromosome 19q13.32|apolipoprotein E|317|lipid transport; the ε4 allele is the main genetic risk factor for late-onset Alzheimer's disease|mouse Apoe (chromosome 7)
human|htt,huntingtin|chromosome 4p16.3|huntingtin|3142|scaffold protein in neurons; a CAG repeat longer than 35 causes Huntington's disease|mouse Htt (chromosome 5)
human|dmd,dystrophin|chromosome Xp21.2–p21.1|dystrophin|3685|links the muscle cytoskeleton to the cell membrane; the largest human gene (about 2.2 Mb); mutations cause Duchenne and Becker muscular dystrophy|mouse Dmd (the mdx mouse)
human|egfr,erbb1,her1|chromosome 7p11.2|epidermal growth factor receptor|1210|receptor tyrosine kinase for EGF; often mutated or amplified in lung cancer and glioblastoma|fruit fly Egfr, nematode let-23, paralogs ERBB2, ERBB3, ERBB4
human|kras|chromosome 12p12.1|GTPase KRas|189|small GTPase in growth-factor signalling; the most commonly mutated oncogene (G12 and G13 hotspots)|mouse Kras, paralogs HRAS and NRAS, fruit fly Ras85D
human|myc,c-myc|chromosome 8q24.21|Myc proto-oncogene protein|439|transcription factor driving cell growth; translocated in Burkitt lymphoma|paralogs MYCN and MYCL, fruit fly dm
human|sry|chromosome Yp11.2|sex-determining region Y protein|204|triggers testis development in the embryo|mouse Sry, SOX family paralogs (SOX9, SOX3)
human|foxp2|chromosome 7q31.1|forkhead box protein P2|715|transcription factor needed for speech and language development|mouse Foxp2, songbird FoxP2, paralogs FOXP1, FOXP4
human|lct,lactase|chromosome 2q21.3|lactase-phlorizin hydrolase|1927|digests lactose in the small intestine; regulatory variants upstream (in MCM6) keep it active in adults (lactase persistence)|mouse Lct
human|mc1r|chromosome 16q24.3|melanocortin 1 receptor|317|switches melanocytes between eumelanin and pheomelanin; variants cause red hair and fair skin|mouse Mc1r (the extension locus)
human|pax6|chromosome 11p13|paired box protein Pax-6|422|master regulator of eye development; mutations cause aniridia|fruit fly eyeless and twin of eyeless, mouse Pax6 (small eye)
human|tert|chromosome 5p15.33|telomerase reverse transcriptase|1132|catalytic subunit of telomerase, which extends telomeres|yeast EST2, mouse Tert
human|gapdh|chromosome 12p13.31|glyceraldehyde-3-phosphate dehydrogenase|335|glycolytic enzyme; a common housekeeping reference gene|E. coli gapA, yeast TDH1–3
human|actb,beta actin|chromosome 7p22.1|actin, cytoplasmic 1 (β-actin)|375|cytoskeleton; a common housekeeping reference gene|yeast ACT1, many actin paralogs
fruit fly|hox,hox genes,homeotic genes,hom-c|chromosome 3R|homeodomain transcription factors|—|set the identity of body segments along the head-to-tail axis. Eight genes in two clusters: the Antennapedia complex (lab, pb, Dfd, Scr, Antp) and the Bithorax complex (Ubx, abd-A, Abd-B); mutations transform one segment into another (e.g. legs in place of antennae)|human HOX genes: 39 genes in four clusters (HOXA on 7p15, HOXB on 17q21, HOXC on 12q13, HOXD on 2q31)
human|hox,hox genes,homeotic genes|four clusters: HOXA 7p15, HOXB 17q21, HOXC 12q13, HOXD 2q31|homeodomain transcription factors|—|39 genes that pattern the body axis and limbs|fruit fly Antennapedia and Bithorax complexes
fruit fly|crmp,collapsin response mediator protein|chromosome 2|collapsin response mediator protein (dihydropyrimidinase)|—|dihydropyrimidinase in pyrimidine breakdown; also needed for normal nervous system development|human DPYS and CRMP1/DPYSL2–5
fruit fly|eyeless,ey|chromosome 4|eyeless|—|master regulator of eye development; misexpression grows eyes on legs and wings|human and mouse PAX6
fruit fly|white,w|X chromosome|ABC transporter white|687|transports eye pigment precursors; the first fly mutation found by Thomas Hunt Morgan (1910)|human ABCG1
";

fn gene_rows() -> &'static Vec<Vec<&'static str>> {
    static G: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    G.get_or_init(|| GENES.lines().map(|l| l.split('|').collect()).collect())
}

fn find_gene(org: Option<&str>, sym: &str) -> Option<&'static Vec<&'static str>> {
    let org = org.map(|o| match o {
        "drosophila" | "fly" => "fruit fly",
        "escherichia coli" | "e coli" | "e.coli" => "e. coli",
        "murine" => "mouse",
        o => o,
    });
    let hit = |r: &&Vec<&str>| r[1].split(',').any(|n| n == sym);
    let rows = gene_rows();
    match org {
        Some(o) => rows.iter().filter(|r| r[0] == o).find(hit),
        None => rows.iter().filter(|r| r[0] == "human").find(hit).or_else(|| rows.iter().find(hit)),
    }
}

fn gene_symbol(r: &[&str]) -> String {
    let s = r[1].split(',').next().unwrap_or("");
    match r[0] {
        "human" => s.to_uppercase(),
        "mouse" => {
            let mut c = s.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        }
        // bacterial genes: three lower-case letters and a capital (mutS)
        "e. coli" if s.len() == 4 => s[..3].to_string() + &s[3..].to_uppercase(),
        _ => s.to_string(),
    }
}

fn gene_label(r: &[&str]) -> String {
    let sym = gene_symbol(r);
    let sym = if r[0] == "e. coli" { sym } else if r[0] == "fruit fly" && sym == "hox" { "Hox".into() } else { sym };
    format!("{} {}", match r[0] { "e. coli" => "E. coli", o => o }, sym)
}

fn genes(l: &str, raw_bare: bool) -> Option<Answer> {
    macro_rules! org {
        () => {
            r"(?:(human|mouse|murine|fruit fly|drosophila|fly|e\. ?coli|e coli|escherichia coli) )?"
        };
    }
    if let Some(m) = re!(concat!(r"^homologs? (?:of|for) (?:the )?", org!(), r"(?:gene )?([a-z0-9-]+)(?: gene)?$")).captures(l) {
        let r = find_gene(m.get(1).map(|x| x.as_str()), &m[2])?;
        return Some(Answer::new("gene homologs", vec![
            input(format!("{} | homologs", gene_label(r))),
            primary(pod("Result", "Homologs", text(r[6]))),
            pod("Gene", "Gene", text(format!("{} — {}", gene_label(r), r[3]))),
        ]));
    }
    let l = l.trim_end_matches(" genes").trim_end_matches(" sequence");
    let parts: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+and\s+)\s*").split(l).collect();
    let item = re!(concat!("^", org!(), r"(?:gene )?([a-z0-9-]+)( gene)?$"));
    let mut rows = Vec::new();
    for p in &parts {
        let m = item.captures(p.trim())?;
        // without an organism or the word gene, accept only distinctive symbols (letters and digits)
        let explicit = m.get(1).is_some() || m.get(3).is_some() || l.contains("gene");
        let sym = &m[2];
        if !explicit && !(raw_bare && sym.len() >= 4 && sym.chars().any(|c| c.is_ascii_digit()) && sym.chars().any(|c| c.is_ascii_alphabetic())) {
            return None;
        }
        rows.push(find_gene(m.get(1).map(|x| x.as_str()), sym)?);
    }
    let card = |r: &[&str]| vec![
        vec![text("organism"), text(match r[0] { "e. coli" => "Escherichia coli", "fruit fly" => "fruit fly (Drosophila melanogaster)", "mouse" => "house mouse (Mus musculus)", _ => "human (Homo sapiens)" })],
        vec![text("location"), text(r[2])],
        vec![text("protein"), text(r[3])],
        vec![text("length"), text(if r[4] == "—" { "—".to_string() } else { format!("{} amino acids", r[4]) })],
        vec![text("function"), text(r[5])],
        vec![text("homologs"), text(r[6])],
    ];
    if rows.len() == 1 {
        let r = rows[0];
        return Some(Answer::new("gene", vec![
            input(format!("{} | gene", gene_label(r))),
            primary(pod("Result", "Protein", text(r[3]))),
            pod("Properties", "Gene", table(None, card(r))),
        ]));
    }
    let mut pods = vec![input(format!("{} | genes", rows.iter().map(|r| gene_label(r)).collect::<Vec<_>>().join(", "))),
        primary(pod("Result", "Proteins", text(rows.iter().map(|r| format!("{}: {}", gene_label(r), r[3])).collect::<Vec<_>>().join(" | "))))];
    if rows.len() == 2 && (rows[0][6].to_lowercase().contains(&gene_symbol(rows[1]).to_lowercase()) || rows[1][6].to_lowercase().contains(&gene_symbol(rows[0]).to_lowercase())) {
        pods.push(pod("Relation", "Relationship", text(format!("{} and {} are homologs", gene_label(rows[0]), gene_label(rows[1])))));
    }
    for r in &rows {
        pods.push(pod("Gene", &gene_label(r), table(None, card(r))));
    }
    Some(Answer::new("genes", pods))
}

fn insulin(l: &str) -> Option<Answer> {
    if !re!(r"^(?:human )?insulin (?:amino acid )?(?:sequence|primary structure|structure)$|^(?:amino acid )?sequence of (?:human )?insulin$").is_match(l) {
        return None;
    }
    let a = "GIVEQCCTSICSLYQLENYCN";
    let b = "FVNQHLCGSHLVEALYLVCGERGFFYTPKT";
    Some(Answer::new("insulin", vec![
        input("human insulin | amino acid sequence"),
        primary(pod("Result", "Sequence", text(format!("A chain: {a} | B chain: {b}")))),
        pod("Properties", "Properties", table(None, vec![
            vec![text("A chain"), text(format!("{} residues", a.len()))],
            vec![text("B chain"), text(format!("{} residues", b.len()))],
            vec![text("disulfide bonds"), text("A7–B7 and A20–B19 between the chains; A6–A11 within the A chain")],
            vec![text("molecular mass"), text("5807.6 g/mol")],
            vec![text("formula"), text("C₂₅₇H₃₈₃N₆₅O₇₇S₆")],
        ])),
        pod("Note", "Note", text("made in pancreatic β cells as proinsulin; the C-peptide is cut out to leave the two chains")),
    ]))
}

/// names | summary | steps | net
const PATHWAYS: &[(&[&str], &str, &str, &str, &str)] = &[
    (&["tca cycle", "citric acid cycle", "krebs cycle", "tricarboxylic acid cycle", "citric acid cycle (tca cycle)"], "citric acid cycle", "oxidizes acetyl-CoA to CO₂ in the mitochondrial matrix",
     "1. citrate synthase: acetyl-CoA + oxaloacetate → citrate; 2. aconitase: citrate → isocitrate; 3. isocitrate dehydrogenase: isocitrate → α-ketoglutarate + CO₂ + NADH; 4. α-ketoglutarate dehydrogenase: → succinyl-CoA + CO₂ + NADH; 5. succinyl-CoA synthetase: → succinate + GTP (or ATP); 6. succinate dehydrogenase: → fumarate + FADH₂; 7. fumarase: → malate; 8. malate dehydrogenase: → oxaloacetate + NADH",
     "per acetyl-CoA: 3 NADH, 1 FADH₂, 1 GTP (ATP), 2 CO₂ — about 10 ATP after oxidative phosphorylation"),
    (&["glycolysis", "embden-meyerhof pathway", "glycolytic pathway"], "glycolysis", "splits glucose into two pyruvate in the cytosol",
     "1. hexokinase: glucose → glucose-6-phosphate (uses ATP); 2. phosphoglucose isomerase: → fructose-6-phosphate; 3. phosphofructokinase-1: → fructose-1,6-bisphosphate (uses ATP; the main control point); 4. aldolase: → dihydroxyacetone phosphate + glyceraldehyde-3-phosphate; 5. triose phosphate isomerase; 6. glyceraldehyde-3-phosphate dehydrogenase: → 1,3-bisphosphoglycerate + NADH; 7. phosphoglycerate kinase: → 3-phosphoglycerate + ATP; 8. phosphoglycerate mutase: → 2-phosphoglycerate; 9. enolase: → phosphoenolpyruvate; 10. pyruvate kinase: → pyruvate + ATP",
     "glucose + 2 NAD⁺ + 2 ADP + 2 Pᵢ → 2 pyruvate + 2 NADH + 2 ATP + 2 H₂O"),
    (&["urea cycle", "ornithine cycle", "krebs-henseleit cycle"], "urea cycle", "converts ammonia to urea in the liver (mitochondria and cytosol)",
     "1. carbamoyl phosphate synthetase I: NH₃ + HCO₃⁻ + 2 ATP → carbamoyl phosphate; 2. ornithine transcarbamylase: + ornithine → citrulline; 3. argininosuccinate synthetase: citrulline + aspartate + ATP → argininosuccinate; 4. argininosuccinate lyase: → arginine + fumarate; 5. arginase: arginine + H₂O → urea + ornithine",
     "NH₃ + HCO₃⁻ + aspartate + 3 ATP → urea + fumarate + 2 ADP + AMP + 4 Pᵢ (4 high-energy phosphate bonds)"),
    (&["glycogen degradation", "glycogen degradation pathway", "glycogenolysis", "glycogen breakdown"], "glycogenolysis", "releases glucose units from glycogen (liver and muscle)",
     "1. glycogen phosphorylase: removes glucose from the non-reducing ends as glucose-1-phosphate (α-1,4 bonds), stopping four residues from a branch; 2. debranching enzyme: transfers three residues (transferase) and hydrolyses the α-1,6 branch glucose (glucosidase) as free glucose; 3. phosphoglucomutase: glucose-1-phosphate → glucose-6-phosphate; 4. in the liver, glucose-6-phosphatase: → free glucose released to the blood (muscle uses glucose-6-phosphate in glycolysis)",
     "stimulated by glucagon and adrenaline (via cAMP and phosphorylase kinase); inhibited by insulin"),
    (&["glycogen synthesis", "glycogenesis"], "glycogenesis", "stores glucose as glycogen",
     "1. hexokinase or glucokinase: glucose → glucose-6-phosphate; 2. phosphoglucomutase: → glucose-1-phosphate; 3. UDP-glucose pyrophosphorylase: → UDP-glucose; 4. glycogen synthase: adds α-1,4-linked glucose to a primer (glycogenin); 5. branching enzyme: makes α-1,6 branches",
     "one UTP (equivalent to one ATP) per glucose stored"),
    (&["calvin cycle", "calvin-benson cycle", "light-independent reactions", "dark reactions"], "Calvin cycle", "fixes CO₂ into sugar in the chloroplast stroma",
     "1. carbon fixation: RuBisCO joins CO₂ to ribulose-1,5-bisphosphate → 2 × 3-phosphoglycerate; 2. reduction: ATP and NADPH convert 3-phosphoglycerate to glyceraldehyde-3-phosphate; 3. regeneration: ribulose-1,5-bisphosphate is rebuilt using ATP",
     "3 CO₂ + 9 ATP + 6 NADPH → 1 glyceraldehyde-3-phosphate + 9 ADP + 8 Pᵢ + 6 NADP⁺"),
    (&["beta oxidation", "β-oxidation", "beta-oxidation", "fatty acid oxidation"], "β-oxidation", "breaks fatty acids into acetyl-CoA in the mitochondria",
     "1. acyl-CoA dehydrogenase: → trans-Δ²-enoyl-CoA + FADH₂; 2. enoyl-CoA hydratase: → L-3-hydroxyacyl-CoA; 3. 3-hydroxyacyl-CoA dehydrogenase: → 3-ketoacyl-CoA + NADH; 4. thiolase: → acetyl-CoA + acyl-CoA two carbons shorter",
     "palmitate (C16): 7 rounds give 8 acetyl-CoA, 7 FADH₂ and 7 NADH — about 106 ATP net"),
    (&["oxidative phosphorylation", "electron transport chain", "respiratory chain"], "oxidative phosphorylation", "makes ATP from NADH and FADH₂ on the inner mitochondrial membrane",
     "1. complex I (NADH dehydrogenase) passes electrons from NADH to ubiquinone, pumping 4 H⁺; 2. complex II (succinate dehydrogenase) passes electrons from FADH₂; 3. complex III (cytochrome bc₁) to cytochrome c, pumping 4 H⁺; 4. complex IV (cytochrome c oxidase) to O₂, forming water and pumping 2 H⁺; 5. ATP synthase uses the proton gradient to make ATP",
     "about 2.5 ATP per NADH and 1.5 ATP per FADH₂"),
];

fn pathways(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("the ").trim_end_matches(" steps").trim_end_matches(" pathway diagram");
    let (_, name, what, steps, net) = PATHWAYS.iter().find(|p| p.0.contains(&l))?;
    let rows: Vec<Vec<Content>> = steps.split("; ").map(|st| {
        let (num, rest) = st.split_once(". ").unwrap_or(("", st));
        vec![text(num), text(rest)]
    }).collect();
    Some(Answer::new(*name, vec![
        input(format!("{name} | metabolic pathway")),
        primary(pod("Result", "Summary", text(*what))),
        pod("Steps", "Steps", table(Some(vec!["", "reaction"]), rows)),
        pod("Net", "Net reaction and yield", text(*net)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).to_string();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what is |what are |what's |show |give me |list )?(?:the )?").replace(&l, "").to_string();
    let bare = !q.raw.trim().contains(' ');
    sequence(q).or_else(|| codons(&l)).or_else(|| genes(&l, bare)).or_else(|| genomes(&l)).or_else(|| insulin(&l)).or_else(|| pathways(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_and_sequences() {
        assert_eq!(codon_table().len(), 64);
        assert_eq!(amino("AUG"), 'M');
        assert_eq!(amino("UGG"), 'W');
        assert_eq!(codon_table().iter().filter(|x| x.1 == '*').count(), 3);
        assert_eq!(codon_table().iter().filter(|x| x.1 == 'L').count(), 6);
        assert_eq!(translate("AUGGCCUAA"), "MA*");
        assert_eq!(complement("ATGC", false), "TACG");
        for line in GENES.lines() {
            assert_eq!(line.split('|').count(), 7, "{line}");
        }
        for line in GENOMES.lines() {
            assert_eq!(line.split('|').count(), 5, "{line}");
        }
    }
}
