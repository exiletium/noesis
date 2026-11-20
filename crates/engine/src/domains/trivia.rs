//! Institutions (universities, foundations, private companies), city
//! elevations, fictional families and creatures, poems on a subject and a
//! few historical questions.
//!
//! "Harvard University, Princeton University", "Gates Foundation, Lilly
//! Endowment", "elevation Denver - Los Angeles", "Snoopy's family",
//! "Gothita, Gothorita, Gothitelle", "poems featuring santa claus".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Inst {
    name: &'static str,
    aliases: &'static [&'static str],
    kind: &'static str,
    founded: &'static str,
    location: &'static str,
    founders: &'static str,
    facts: &'static [(&'static str, &'static str)],
}

const INSTITUTIONS: &[Inst] = &[
    Inst { name: "New York University", aliases: &["nyu", "new york university"], kind: "private research university", founded: "1831", location: "Manhattan, New York City", founders: "Albert Gallatin and others", facts: &[("students", "about 59,000 (2023)"), ("campuses", "New York City, Abu Dhabi and Shanghai"), ("colors", "violet")] },
    Inst { name: "Harvard University", aliases: &["harvard", "harvard university", "harvard college"], kind: "private Ivy League research university", founded: "1636", location: "Cambridge, Massachusetts", founders: "Massachusetts Bay Colony (named after John Harvard)", facts: &[("students", "about 25,000 (about 7,000 undergraduates)"), ("endowment", "about $53 billion (2024), the largest of any university"), ("colors", "crimson"), ("note", "the oldest institution of higher learning in the United States")] },
    Inst { name: "Princeton University", aliases: &["princeton", "princeton university"], kind: "private Ivy League research university", founded: "1746 (as the College of New Jersey)", location: "Princeton, New Jersey", founders: "New Light Presbyterians", facts: &[("students", "about 9,000 (about 5,600 undergraduates)"), ("endowment", "about $34 billion (2024)"), ("colors", "orange and black"), ("note", "the fourth-oldest college in the United States")] },
    Inst { name: "Ford Foundation", aliases: &["ford foundation", "the ford foundation"], kind: "private foundation", founded: "January 15, 1936", location: "New York City", founders: "Edsel Ford and Henry Ford", facts: &[("focus", "reducing inequality; social justice, democracy and the arts"), ("endowment", "about $16 billion")] },
    Inst { name: "Gates Foundation", aliases: &["gates foundation", "bill & melinda gates foundation", "bill and melinda gates foundation", "the gates foundation"], kind: "private foundation", founded: "2000", location: "Seattle, Washington", founders: "Bill Gates and Melinda French Gates", facts: &[("focus", "global health, poverty and education"), ("endowment", "about $75 billion (2024), one of the largest private foundations"), ("note", "renamed the Gates Foundation in 2024; it plans to close by 2045")] },
    Inst { name: "Lilly Endowment", aliases: &["lilly endowment", "the lilly endowment"], kind: "private foundation", founded: "1937", location: "Indianapolis, Indiana", founders: "J. K. Lilly Sr. and his sons Eli Lilly and J. K. Lilly Jr.", facts: &[("focus", "community development, education and religion, mainly in Indiana"), ("assets", "largely Eli Lilly and Company stock")] },
    Inst { name: "Schneider National", aliases: &["schneider national", "schneider"], kind: "trucking and logistics company (NYSE: SNDR)", founded: "1935", location: "Green Bay, Wisconsin", founders: "Al Schneider", facts: &[("industry", "truckload, intermodal and logistics services"), ("note", "known for its orange trucks")] },
    Inst { name: "Koch Industries", aliases: &["koch industries", "koch"], kind: "privately held conglomerate", founded: "1940 (as Wood River Oil & Refining Company)", location: "Wichita, Kansas", founders: "Fred C. Koch", facts: &[("industries", "refining, chemicals, pulp and paper (Georgia-Pacific), fertilizers, electronics (Molex), ranching"), ("note", "one of the largest privately held companies in the United States; led by Charles Koch")] },
    Inst { name: "J.R. Simplot Company", aliases: &["j.r. simplot", "jr simplot", "j. r. simplot", "j.r. simplot company", "simplot"], kind: "privately held agribusiness company", founded: "1929", location: "Boise, Idaho", founders: "J. R. Simplot", facts: &[("industries", "frozen potato products, fertilizer, cattle, food processing"), ("note", "supplied McDonald's with frozen french fries from the 1960s")] },
];

/// City elevations, metres (approximate, at the city centre or official reference point).
const ELEVATIONS: &[(&str, f64)] = &[
    ("denver", 1609.0), ("los angeles", 93.0), ("chicago", 181.0), ("new york", 10.0), ("new york city", 10.0), ("salt lake city", 1288.0), ("albuquerque", 1619.0), ("phoenix", 331.0), ("las vegas", 610.0), ("atlanta", 320.0), ("dallas", 131.0), ("houston", 15.0), ("miami", 2.0), ("seattle", 53.0), ("san francisco", 16.0), ("boston", 43.0), ("el paso", 1140.0), ("colorado springs", 1839.0), ("mexico city", 2240.0), ("bogota", 2640.0), ("bogotá", 2640.0), ("quito", 2850.0), ("la paz", 3640.0), ("cusco", 3400.0), ("addis ababa", 2355.0), ("nairobi", 1795.0), ("johannesburg", 1753.0), ("kathmandu", 1400.0), ("lhasa", 3650.0), ("tehran", 1190.0), ("madrid", 667.0), ("paris", 35.0), ("london", 11.0), ("amsterdam", -2.0), ("berlin", 34.0),
];

fn elevation(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(?:elevation|altitude|height above sea level) (?:of )?(.+)$|^how high (?:is|above sea level is) (.+)$|^(.+?) (?:elevation|altitude)$").captures(l)?;
    let s = m.get(1).or(m.get(2)).or(m.get(3))?.as_str().trim();
    let s = re!(r",? (?:co|colorado|ca|california|il|illinois|ny|usa|us)$").replace(s, "").to_string();
    let el = |n: &str| ELEVATIONS.iter().find(|(c, _)| *c == n.trim()).map(|x| x.1);
    let cap = |n: &str| n.split(' ').map(|w| {
        let mut c = w.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    }).collect::<Vec<_>>().join(" ");
    let ft = |m: f64| (m * 3.28084).round();
    // "elevation Denver - Los Angeles"
    if let Some(d) = re!(r"^(.+?) (?:-|minus) (.+)$").captures(&s) {
        let (a, b) = (el(&d[1])?, el(&d[2])?);
        return Some(Answer::new("elevation", vec![
            pod("Input", "Input interpretation", text(format!("elevation of {} − elevation of {}", cap(&d[1]), cap(&d[2])))),
            primary(pod("Result", "Result", text(format!("about {} m ({} ft)", a - b, ft(a - b))))),
            pod("Values", "Elevations", table(None, vec![vec![text(cap(&d[1])), text(format!("about {a} m ({} ft)", ft(a)))], vec![text(cap(&d[2])), text(format!("about {b} m ({} ft)", ft(b)))]])),
        ]));
    }
    let a = el(&s)?;
    Some(Answer::new("elevation", vec![
        pod("Input", "Input interpretation", text(format!("{} | elevation", cap(&s)))),
        primary(pod("Result", "Elevation", text(format!("about {a} m ({} ft) above sea level", ft(a))))),
        pod("Note", "Note", text("at the city centre or official reference point; elevations vary across the city")),
    ]))
}

fn institutions(l: &str) -> Option<Answer> {
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?(?:\s|$)|\bversus\b)\s*").split(l).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let v: Option<Vec<&Inst>> = parts.iter().map(|p| INSTITUTIONS.iter().find(|i| i.aliases.contains(&p.trim()))).collect();
    let v = v?;
    // bare one-word names need the full form
    if v.len() == 1 && matches!(l, "harvard" | "princeton" | "koch" | "simplot" | "schneider") {
        return None;
    }
    if v.len() == 1 {
        let i = v[0];
        let mut rows = vec![vec![text("type"), text(i.kind)], vec![text("founded"), text(i.founded)], vec![text("location"), text(i.location)], vec![text("founders"), text(i.founders)]];
        rows.extend(i.facts.iter().map(|(k, v)| vec![text(*k), text(*v)]));
        return Some(Answer::new("institution", vec![
            pod("Input", "Input interpretation", text(i.name)),
            primary(pod("Result", "Basic information", text(format!("{} in {}, founded {}", i.kind, i.location, i.founded)))),
            pod("Table", "Details", table(None, rows)),
        ]));
    }
    let rows = v.iter().map(|i| vec![text(i.name), text(i.kind), text(i.founded), text(i.location)]).collect();
    Some(Answer::new("institutions", vec![
        pod("Input", "Input interpretation", text(v.iter().map(|i| i.name).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", "Comparison", text(v.iter().map(|i| format!("{} (founded {}, {})", i.name, i.founded, i.location)).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Details", table(Some(vec!["name", "type", "founded", "location"]), rows)),
    ]))
}

/// Pokémon and Digimon with their types and evolutions.
const CREATURES: &[(&str, &str, &[(&str, &str)])] = &[
    ("gothita", "Gothita (Pokémon #574)", &[("type", "Psychic"), ("evolves into", "Gothorita at level 32"), ("introduced", "Pokémon Black and White (generation V, 2010)")]),
    ("gothorita", "Gothorita (Pokémon #575)", &[("type", "Psychic"), ("evolves from", "Gothita at level 32"), ("evolves into", "Gothitelle at level 41"), ("introduced", "Pokémon Black and White (generation V, 2010)")]),
    ("gothitelle", "Gothitelle (Pokémon #576)", &[("type", "Psychic"), ("evolves from", "Gothorita at level 41"), ("introduced", "Pokémon Black and White (generation V, 2010)")]),
    ("gomamon", "Gomamon (Digimon)", &[("level", "Rookie (Child)"), ("type", "Sea Animal"), ("attribute", "Vaccine"), ("digivolves into", "Ikkakumon"), ("partner", "Joe Kido in Digimon Adventure (1999)")]),
    ("metalgreymon", "MetalGreymon (Digimon)", &[("level", "Ultimate (Perfect)"), ("type", "Cyborg"), ("attribute", "Vaccine (a Virus-attribute variant also exists)"), ("digivolves from", "Greymon"), ("digivolves into", "WarGreymon"), ("partner", "Tai Kamiya's Agumon line in Digimon Adventure")]),
];

fn creatures(l: &str) -> Option<Answer> {
    let l = re!(r"^(?:what is )?(.+?)'s type$").replace(l, "$1 type").to_string();
    let l = l.trim_end_matches(" type").trim_end_matches(" pokemon").trim_end_matches(" digimon").replace("metal greymon", "metalgreymon");
    let parts: Vec<&str> = re!(r"\s*(?:,|\band\b)\s*").split(&l).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let v: Option<Vec<&(&str, &str, &[(&str, &str)])>> = parts.iter().map(|p| CREATURES.iter().find(|c| c.0 == p.trim())).collect();
    let v = v?;
    let main = v.iter().map(|c| format!("{}: {}", c.1, c.2.iter().find(|r| r.0 == "type").map(|r| r.1).unwrap_or(""))).collect::<Vec<_>>().join(" | ");
    let mut pods = vec![pod("Input", "Input interpretation", text(v.iter().map(|c| c.1).collect::<Vec<_>>().join(" | "))), primary(pod("Result", "Type", text(main)))];
    for c in &v {
        pods.push(pod("Details", c.1, table(None, c.2.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect())));
    }
    Some(Answer::new("fictional creature", pods))
}

fn families(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(?:family (?:of|members of) |relatives of )?(snoopy|jon snow)(?:'s)?(?: family| relatives| siblings)?$").captures(l)?;
    if !(l.contains("family") || l.contains("relatives") || l.contains("siblings")) {
        return None;
    }
    let (title, main, rows): (&str, &str, Vec<(&str, &str)>) = match &m[1] {
        "snoopy" => ("Snoopy (Peanuts) | family", "siblings Spike, Belle, Marbles, Olaf, Andy, Molly and Rover", vec![("siblings", "Spike, Belle, Marbles, Olaf, Andy, Molly, Rover (eight puppies in all, including Snoopy)"), ("birthplace", "Daisy Hill Puppy Farm"), ("nephew", "Belle's son"), ("owner", "Charlie Brown")]),
        _ => ("Jon Snow (A Song of Ice and Fire / Game of Thrones) | family", "raised by Eddard Stark as his illegitimate son; his parents are Lyanna Stark and Rhaegar Targaryen (revealed in the TV series)", vec![("mother", "Lyanna Stark"), ("father", "Rhaegar Targaryen"), ("raised by", "Eddard \"Ned\" Stark (his uncle)"), ("cousins raised as half-siblings", "Robb, Sansa, Arya, Bran and Rickon Stark"), ("aunt", "Daenerys Targaryen")]),
    };
    Some(Answer::new("fictional character", vec![
        pod("Input", "Input interpretation", text(title)),
        primary(pod("Result", "Family", text(main))),
        pod("Table", "Relatives", table(None, rows.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect())),
    ]))
}

fn poems(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:name (?:the )?)?poems (?:featuring|about|on|mentioning|with) (?:the )?(santa claus|saint nicholas|st\.? nicholas|count roland|roland)$").captures(l)?;
    let (subject, rows): (&str, Vec<(&str, &str, &str)>) = if m[1].contains("roland") {
        ("Roland", vec![("The Song of Roland (La Chanson de Roland)", "anonymous (attributed to Turold)", "c. 1100"), ("Morgante", "Luigi Pulci", "1478–1483"), ("Orlando Innamorato", "Matteo Maria Boiardo", "1483–1495"), ("Orlando Furioso", "Ludovico Ariosto", "1516–1532")])
    } else {
        ("Santa Claus", vec![("A Visit from St. Nicholas (\"'Twas the Night Before Christmas\")", "Clement Clarke Moore (attributed)", "1823"), ("Old Santeclaus with Much Delight", "anonymous", "1821")])
    };
    Some(Answer::new("poems", vec![
        pod("Input", "Input interpretation", text(format!("poems about {subject}"))),
        primary(pod("Result", "Poems", text(rows.iter().map(|r| r.0).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Poems", table(Some(vec!["poem", "author", "date"]), rows.iter().map(|(a, b, c)| vec![text(*a), text(*b), text(*c)]).collect())),
    ]))
}

fn history(l: &str) -> Option<Answer> {
    let one = |inp: &str, main: &str, rows: Vec<(&str, &str)>| {
        Some(Answer::new("history", vec![
            pod("Input", "Input interpretation", text(inp.to_string())),
            primary(pod("Result", "Result", text(main.to_string()))),
            pod("Table", "Details", table(None, rows.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect())),
        ]))
    };
    if re!(r"^(?:when did )?(?:the )?codices replace scrolls$|^when did (?:the )?codex replace (?:the )?scroll$").is_match(l) {
        return one("codex replacing the scroll", "gradually between the 1st and 5th centuries AD; codices outnumbered scrolls by about AD 300–400", vec![("early use", "Roman writers (Martial, late 1st century AD) mention parchment codices"), ("drivers", "early Christians favoured the codex for scripture"), ("by about AD 500", "the scroll had largely fallen out of use for books")]);
    }
    if re!(r"^diet of worms, vaccinations? (?:invented|invention), spam debuts?$").is_match(l) {
        return one("Diet of Worms | invention of vaccination | debut of Spam", "1521 | 1796 | 1937", vec![("Diet of Worms", "January 28 – May 25, 1521; Martin Luther refused to recant and was declared an outlaw (Edict of Worms)"), ("vaccination", "May 14, 1796: Edward Jenner inoculated James Phipps with cowpox to protect against smallpox"), ("Spam (canned meat)", "introduced by Hormel Foods on July 5, 1937"), ("spam (email)", "the first mass unsolicited email was sent on ARPANET by Gary Thuerk on May 3, 1978")]);
    }
    if re!(r"^(?:charles )?lindbergh(?:'s)? (?:solo |transatlantic )?flight (?:cities|places|locations)(?: involved)?$|^cities involved in lindbergh(?:'s)? (?:solo )?flight$").is_match(l) {
        return one("Charles Lindbergh's solo transatlantic flight | cities", "New York (Roosevelt Field, Garden City, Long Island) to Paris (Le Bourget Field)", vec![("date", "May 20–21, 1927"), ("aircraft", "Spirit of St. Louis (Ryan NYP)"), ("duration", "33.5 hours, about 5,800 km (3,600 mi)"), ("earlier legs", "San Diego → St. Louis → New York, May 10–12, 1927"), ("prize", "the $25,000 Orteig Prize")]);
    }
    if re!(r"^(?:people|signers|signatories) (?:at|of) the signing of the declaration of independence$|^(?:signers|signatories) of the declaration of independence$|^who signed the declaration of independence$").is_match(l) {
        return one("signers of the US Declaration of Independence", "56 delegates of the Second Continental Congress, including John Hancock, Thomas Jefferson, John Adams, Benjamin Franklin, Samuel Adams, Roger Sherman, Robert Morris and Benjamin Rush", vec![("number of signers", "56"), ("first signer", "John Hancock, President of the Congress"), ("main signing", "August 2, 1776, in Philadelphia (Independence Hall); some signed later"), ("Committee of Five", "Thomas Jefferson (principal author), John Adams, Benjamin Franklin, Roger Sherman, Robert R. Livingston (who did not sign)"), ("oldest signer", "Benjamin Franklin (70)"), ("youngest signer", "Edward Rutledge (26)")]);
    }
    None
}

/// Short definitions: (names, title, definition, details).
const DEFS: &[(&[&str], &str, &str, &[(&str, &str)])] = &[
    (&["hierarchy of icd-10 s52.331g", "icd-10 s52.331g", "s52.331g", "icd-10 code s52.331g", "icd10 s52.331g"], "ICD-10-CM S52.331G", "displaced oblique fracture of shaft of right radius, subsequent encounter for closed fracture with delayed healing", &[("chapter 19", "Injury, poisoning and certain other consequences of external causes (S00–T88)"), ("block", "S50–S59 Injuries to the elbow and forearm"), ("S52", "fracture of forearm"), ("S52.3", "fracture of shaft of radius"), ("S52.33", "oblique fracture of shaft of radius"), ("S52.331", "displaced oblique fracture of shaft of right radius"), ("7th character G", "subsequent encounter for closed fracture with delayed healing")]),
    (&["memristance", "magnetic memristance", "memristor", "memristors"], "memristance", "the property of a memristor: the rate of change of magnetic flux linkage with charge, M(q) = dΦ/dq, measured in ohms", &[("defining relation", "dΦ = M dq, so v(t) = M(q(t)) i(t): a resistance that depends on the charge that has passed"), ("proposed by", "Leon Chua, 1971, as the fourth basic passive circuit element alongside resistance, capacitance and inductance"), ("first device", "a titanium dioxide memristor reported by HP Labs in 2008"), ("signature", "a pinched hysteresis loop in the current–voltage plane"), ("uses", "non-volatile memory (ReRAM) and neuromorphic computing")]),
    (&["maasai mara", "maasai mara game reserve", "maasai mara national reserve", "masai mara", "what type of reserve is maasai mara game reserve", "maasai mara reserve type"], "Maasai Mara National Reserve", "a national game reserve (protected wildlife area) in Narok County, Kenya", &[("type", "national reserve (game reserve), managed by the Narok County government; not a national park, so Maasai herding continues around it"), ("area", "about 1,510 km² (583 sq mi)"), ("established", "1961 as a wildlife sanctuary"), ("borders", "Serengeti National Park, Tanzania, to the south"), ("known for", "the annual wildebeest migration (July to October) and the Big Five")]),
    (&["captcha", "what is a captcha", "captcha test"], "CAPTCHA", "Completely Automated Public Turing test to tell Computers and Humans Apart: a challenge that is easy for people and hard for programs", &[("coined", "2003, by Luis von Ahn, Manuel Blum, Nicholas Hopper and John Langford (Carnegie Mellon University)"), ("forms", "distorted text, image selection, audio, and behavioural checks (reCAPTCHA v3, invisible challenges)"), ("reCAPTCHA", "used scanned book words to help digitize texts; acquired by Google in 2009"), ("purpose", "blocks automated sign-ups, spam and scraping")]),
    (&["skype", "skype software"], "Skype", "a voice-over-IP application for video calls, voice calls and instant messaging", &[("released", "August 2003"), ("founders", "Niklas Zennström and Janus Friis, with software by Ahti Heinla, Priit Kasesalu and Jaan Tallinn (Estonia)"), ("owners", "eBay (2005), an investor group (2009), Microsoft (2011, about $8.5 billion)"), ("retired", "May 5, 2025, replaced by Microsoft Teams"), ("platforms", "Windows, macOS, Linux, Android, iOS and web")]),
    (&["google.com", "www.google.com", "google website"], "google.com", "the website of Google Search, the most visited website in the world", &[("owner", "Google LLC (Alphabet Inc.), Mountain View, California"), ("domain registered", "September 15, 1997"), ("launched", "Google Search launched in 1998 (Larry Page and Sergey Brin)"), ("type", "web search engine")]),
    (&["myoglobin", "myoglobin protein"], "myoglobin", "an oxygen-storage protein of heart and skeletal muscle, a single globin chain carrying one heme group", &[("length", "154 amino acids (human, including the initial methionine)"), ("mass", "about 17 kDa"), ("gene", "MB, chromosome 22q12.3"), ("structure", "eight α-helices (A–H); the first protein structure solved by X-ray crystallography (John Kendrew, 1958, sperm whale myoglobin)"), ("oxygen binding", "hyperbolic curve, higher affinity than hemoglobin (P50 about 2.8 mmHg)")]),
    (&["estrogen receptor", "estrogen receptors", "oestrogen receptor", "estrogen receptor alpha", "er alpha"], "estrogen receptor", "a nuclear hormone receptor: a ligand-activated transcription factor that binds estradiol and regulates target genes", &[("forms", "ERα (gene ESR1, chromosome 6q25.1, 595 amino acids) and ERβ (gene ESR2, chromosome 14q23.2, 530 amino acids)"), ("domains", "N-terminal activation function AF-1, DNA-binding zinc-finger domain, hinge, ligand-binding domain with AF-2"), ("ligands", "estradiol, estrone, estriol; drugs tamoxifen and raloxifene (modulators) and fulvestrant (degrader)"), ("medicine", "estrogen-receptor status guides breast cancer treatment")]),
    (&["temperature of the solar wind", "solar wind temperature"], "solar wind | temperature", "about 10⁵ K near Earth (typical proton temperature about 1.2 × 10⁵ K at 1 AU)", &[("electron temperature", "about 1.4 × 10⁵ K at 1 AU"), ("speed", "about 300–800 km/s"), ("source", "the solar corona, at about 1–2 million K"), ("trend", "the wind cools as it expands outward, roughly as r^(−0.7) for protons")]),
    (&["solar wind"], "solar wind", "a stream of charged particles (mostly protons and electrons) flowing outward from the Sun's corona", &[("temperature", "about 10⁵ K near Earth (typical proton temperature about 1.2 × 10⁵ K at 1 AU; electrons about 1.4 × 10⁵ K)"), ("speed", "about 300–800 km/s (slow wind about 400 km/s, fast wind about 750 km/s)"), ("density at 1 AU", "about 5 particles per cm³"), ("source temperature", "the corona is about 1–2 million K")]),
    (&["southern oscillation index"], "Southern Oscillation Index", "a standardized measure of the difference in sea-level air pressure between Tahiti and Darwin, Australia", &[("sustained negative values", "El Niño conditions (weaker trade winds, warm central and eastern Pacific)"), ("sustained positive values", "La Niña conditions"), ("published by", "the Australian Bureau of Meteorology (Troup SOI) and NOAA")]),
    (&["polar covalent bonds", "polar covalent bond", "polar bond", "polar bonds"], "polar covalent bond", "a covalent bond in which the shared electrons are held unequally because the two atoms differ in electronegativity", &[("electronegativity difference", "roughly 0.4 to 1.7 on the Pauling scale (larger differences give ionic bonds)"), ("result", "partial charges δ+ and δ− and a bond dipole moment"), ("examples", "O–H in water, H–Cl, C–O, N–H")]),
    (&["hydrogenation reactions", "hydrogenation reaction", "hydrogenation"], "hydrogenation", "a reaction that adds molecular hydrogen (H₂) across a multiple bond, usually with a metal catalyst", &[("catalysts", "nickel, palladium or platinum (heterogeneous); Wilkinson's catalyst (homogeneous)"), ("example", "CH₂=CH₂ + H₂ → CH₃CH₃ (ethene to ethane)"), ("uses", "hardening vegetable oils to margarine, petroleum refining, fine chemicals"), ("reverse", "dehydrogenation")]),
    (&["mom ether", "methoxymethyl ether", "mom group", "mom protecting group"], "methoxymethyl (MOM) ether", "a protecting group for alcohols: R–O–CH₂–O–CH₃", &[("installed with", "chloromethyl methyl ether (MOMCl) and a hindered base such as N,N-diisopropylethylamine, or dimethoxymethane with acid"), ("stable to", "bases, nucleophiles, organometallics, many oxidants and reductants"), ("removed with", "acid (for example HCl in methanol) or Lewis acids")]),
    (&["phosphatidylcholine", "phosphatidylcholines", "lecithin"], "phosphatidylcholine", "a class of phospholipids with a choline head group; a major component of cell membranes", &[("structure", "glycerol esterified with two fatty acids and a phosphocholine group"), ("common name", "lecithin (in foods and supplements)"), ("sources", "egg yolk, soybeans"), ("role", "membrane structure, lung surfactant (dipalmitoylphosphatidylcholine), bile")]),
    (&["sensory organs", "sense organs", "the sense organs"], "sense organs", "organs whose receptors respond to stimuli and send signals to the nervous system", &[("eyes", "vision (rods and cones in the retina)"), ("ears", "hearing (cochlea) and balance (vestibular system)"), ("nose", "smell (olfactory epithelium)"), ("tongue", "taste (taste buds)"), ("skin", "touch, pressure, temperature and pain")]),
    (&["space weather"], "space weather", "conditions in the space environment, driven by the Sun, that can affect satellites, communications, power grids and astronauts", &[("drivers", "solar flares, coronal mass ejections, high-speed solar wind streams and solar energetic particles"), ("effects", "geomagnetic storms, aurorae, radio blackouts, satellite drag and radiation hazards"), ("scales", "NOAA G (geomagnetic storms), S (radiation storms) and R (radio blackouts), each 1–5"), ("index", "the planetary Kp index measures geomagnetic disturbance, 0–9")]),
];

fn defs(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("define ").trim_start_matches("what are ").trim_start_matches("what is ");
    let (_, title, def, rows) = DEFS.iter().find(|d| d.0.contains(&l))?;
    Some(Answer::new("definition", vec![
        pod("Input", "Input interpretation", text(*title)),
        primary(pod("Result", "Definition", text(*def))),
        pod("Table", "Details", table(None, rows.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect())),
    ]))
}

/// Lists: (names, title, &[(item, description)]).
const LISTS: &[(&[&str], &str, &[(&str, &str)])] = &[
    (&["plasma physical effects", "plasma effects", "physical effects in plasmas"], "physical effects in plasmas", &[("Debye shielding", "charges are screened beyond the Debye length"), ("plasma oscillation", "electron density oscillates at the plasma frequency (Langmuir waves)"), ("pinch effect", "a current-carrying plasma is compressed by its own magnetic field"), ("Landau damping", "collisionless damping of plasma waves by resonant particles"), ("magnetic reconnection", "magnetic field lines break and reconnect, releasing energy (solar flares)"), ("Alfvén waves", "transverse waves travelling along magnetic field lines"), ("ambipolar diffusion", "electrons and ions diffuse together, held by their mutual electric field")]),
    (&["paradoxical physical effect", "paradoxical physical effects", "paradoxical effects in physics"], "paradoxical physical effects", &[("Mpemba effect", "hot water sometimes appears to freeze faster than cold water (disputed)"), ("Brazil nut effect", "shaking a mixture of grains brings the largest particles to the top"), ("tea leaf paradox", "stirred tea leaves gather at the centre of the cup, not the edge"), ("Braess's paradox", "adding a road to a network can slow everyone's travel"), ("Feynman's reverse sprinkler", "a lawn sprinkler sucking water in barely turns, and in the reverse sense"), ("Olbers' paradox", "an infinite, eternal, static universe would have a bright night sky"), ("twin paradox", "a travelling twin returns younger than the twin who stayed home"), ("Ehrenfest paradox", "the circumference of a rotating disk in special relativity")]),
    (&["electric resistance physical effects", "electrical resistance physical effects", "resistance effects", "physical effects of electric resistance"], "physical effects involving electrical resistance", &[("Joule heating", "a current through a resistance dissipates power I²R as heat"), ("Hall effect", "a transverse voltage in a conductor carrying current in a magnetic field"), ("magnetoresistance", "resistance changes in a magnetic field; giant magnetoresistance (Fert and Grünberg, 1988) is used in disk read heads"), ("Kondo effect", "a resistance minimum at low temperature caused by magnetic impurities"), ("superconductivity", "resistance vanishes below a critical temperature"), ("skin effect", "alternating current crowds near a conductor's surface, raising its resistance"), ("piezoresistive effect", "mechanical strain changes resistance (strain gauges)"), ("quantum Hall effect", "Hall resistance is quantized in steps of h/(ν e²)")]),
    (&["frequency allocations 900mhz", "frequency allocations 900 mhz", "900 mhz band", "900mhz band", "900 mhz frequency allocations"], "radio frequency allocations near 900 MHz", &[("902–928 MHz", "ISM band in ITU Region 2 (the Americas): unlicensed devices such as RFID, LoRa, cordless phones and amateur radio (33 cm band)"), ("880–915 / 925–960 MHz", "GSM-900 (E-GSM) mobile uplink and downlink in Europe, Africa and Asia"), ("824–849 / 869–894 MHz", "US cellular band (850 MHz)"), ("863–870 MHz", "short-range devices band in Europe (868 MHz)"), ("896–901 / 935–940 MHz", "US land mobile radio (900 MHz SMR / broadband)")]),
];

/// Plants and famous animals: (names, title, &[(property, value)]).
const LIVING: &[(&[&str], &str, &[(&str, &str)])] = &[
    (&["osmunda regalis", "royal fern"], "Osmunda regalis (royal fern)", &[("family", "Osmundaceae"), ("type", "deciduous perennial fern"), ("size", "fronds up to about 2 m (occasionally 3 m)"), ("habitat", "bogs, wet woods and stream banks"), ("range", "Europe, Africa and Asia (with close relatives in the Americas)")]),
    (&["musk rose", "rosa moschata"], "Rosa moschata (musk rose)", &[("family", "Rosaceae"), ("type", "climbing species rose"), ("flowers", "single white flowers with a musky scent, late in the season"), ("origin", "uncertain; long cultivated in western Asia and the Mediterranean"), ("note", "a parent of the Noisette and Hybrid Musk roses")]),
    (&["eastern purple coneflower", "echinacea purpurea", "purple coneflower"], "Echinacea purpurea (eastern purple coneflower)", &[("family", "Asteraceae"), ("type", "herbaceous perennial"), ("height", "about 0.6–1.2 m"), ("flowers", "purple-pink ray florets around a spiny orange-brown cone, summer to autumn"), ("native range", "eastern and central North America")]),
    (&["water horsetail", "equisetum fluviatile"], "Equisetum fluviatile (water horsetail)", &[("family", "Equisetaceae"), ("type", "perennial emergent aquatic plant"), ("stems", "hollow, jointed, up to about 1.5 m"), ("habitat", "shallow water of ponds, lakes and marshes"), ("range", "circumboreal: Europe, northern Asia and North America")]),
    (&["cleome hassleriana", "spider flower", "cleome"], "Cleome hassleriana (spider flower)", &[("family", "Cleomaceae"), ("life cycle", "annual (it is not a perennial; it often self-seeds)"), ("height", "about 1–1.5 m"), ("flowers", "pink, purple or white, with long stamens"), ("native range", "South America (southern Brazil, Paraguay, Argentina)")]),
    (&["rin tin tin", "rin-tin-tin"], "Rin Tin Tin (dog)", &[("breed", "German Shepherd"), ("born", "1918, found as a puppy on a First World War battlefield in Lorraine, France"), ("died", "August 10, 1932"), ("owner", "Lee Duncan, a US Army corporal"), ("career", "starred in 27 Hollywood films and helped make Warner Bros. successful")]),
    (&["pando", "largest famous organism by weight", "largest organism by weight", "heaviest organism"], "Pando (quaking aspen clone)", &[("species", "Populus tremuloides (quaking aspen)"), ("location", "Fishlake National Forest, Utah"), ("mass", "about 6,000 tonnes, the heaviest known organism"), ("area", "about 43 hectares (106 acres), about 47,000 stems from one root system"), ("rival", "the Armillaria ostoyae \"humongous fungus\" in Oregon covers a larger area")]),
];

fn living(l: &str) -> Option<Answer> {
    // "is Cleome hassleriana a perennial"
    if let Some(m) = re!(r"^is (.+?) an? (perennial|annual|biennial)$").captures(l) {
        let (_, title, rows) = LIVING.iter().find(|x| x.0.contains(&m[1].trim()))?;
        let life = rows.iter().find(|r| r.0 == "life cycle" || r.0 == "type")?.1;
        let yes = life.starts_with(&m[2]) || life.contains(&format!(" {}", &m[2])) && !life.contains(&format!("not a {}", &m[2]));
        return Some(Answer::new("plant", vec![
            pod("Input", "Input interpretation", text(format!("{title} | {}?", &m[2]))),
            primary(pod("Result", "Result", text(format!("{} ({life})", if yes { "yes" } else { "no" })))),
            pod("Table", "Details", table(None, rows.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())),
        ]));
    }
    // "compare Solanum and Cucurbitaceae"
    if re!(r"^(?:compare )?solanum (?:and|vs\.?|versus) cucurbitaceae$").is_match(l) {
        return Some(Answer::new("plant taxa", vec![
            pod("Input", "Input interpretation", text("Solanum (genus) vs Cucurbitaceae (family)")),
            primary(pod("Result", "Comparison", text("Solanum: a genus of about 1,500 species in the nightshade family Solanaceae | Cucurbitaceae: the gourd family, about 95 genera and 965 species"))),
            pod("Table", "Details", table(Some(vec!["", "Solanum", "Cucurbitaceae"]), vec![
                vec![text("rank"), text("genus"), text("family")],
                vec![text("parent taxon"), text("Solanaceae (order Solanales)"), text("order Cucurbitales")],
                vec![text("examples"), text("potato, tomato, eggplant, black nightshade"), text("cucumber, melons, watermelon, squash, pumpkin, gourds")],
                vec![text("growth habit"), text("herbs, shrubs, vines and small trees"), text("mostly tendril-climbing or trailing vines")],
            ])),
        ]));
    }
    // "owner of Trigger the horse and Unsinkable Sam"
    if re!(r"^owners? of trigger(?: the horse)? and unsinkable sam$").is_match(l) {
        return Some(Answer::new("famous animals", vec![
            pod("Input", "Input interpretation", text("Trigger (horse) | Unsinkable Sam (cat) | owner")),
            primary(pod("Result", "Owners", text("Trigger: Roy Rogers | Unsinkable Sam: no single owner (a ship's cat)"))),
            pod("Table", "Details", table(None, vec![
                vec![text("Trigger"), text("palomino horse ridden by the cowboy actor Roy Rogers in films and on television (1938–1965)")],
                vec![text("Unsinkable Sam"), text("a cat said to have survived the sinking of the German battleship Bismarck, then HMS Cossack and HMS Ark Royal in 1941")],
            ])),
        ]));
    }
    // "Carpet wool sheep"
    if re!(r"^carpet[- ]wool sheep(?: breeds)?$|^sheep breeds for carpet wool$").is_match(l) {
        let rows = [("Drysdale", "New Zealand, developed from 1929 for carpet wool"), ("Tukidale", "New Zealand"), ("Elliottdale", "Australia"), ("Scottish Blackface", "United Kingdom"), ("Herdwick", "Lake District, England"), ("Swaledale", "northern England"), ("Karakul", "Central Asia")];
        return Some(Answer::new("sheep breeds", vec![
            pod("Input", "Input interpretation", text("sheep breeds kept for carpet wool")),
            primary(pod("Result", "Breeds", text(rows.iter().map(|r| r.0).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Breeds", table(Some(vec!["breed", "origin"]), rows.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())),
            pod("Note", "Note", text("carpet wools are coarse (about 35 μm or more) and often contain hollow medullated fibres")),
        ]));
    }
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?(?:\s|$)|\bversus\b)\s*").split(l).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let v: Vec<&(&[&str], &str, &[(&str, &str)])> = parts.iter().map(|p| LIVING.iter().find(|s| s.0.contains(&p.trim()))).collect::<Option<Vec<_>>>()?;
    if v.len() == 1 && matches!(l, "cleome" | "pando") {
        return None;
    }
    let mut pods = vec![
        pod("Input", "Input interpretation", text(v.iter().map(|s| s.1).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", "Result", text(v.iter().map(|s| format!("{}: {}", s.1, s.2[0].1)).collect::<Vec<_>>().join(" | ")))),
    ];
    for s in &v {
        pods.push(pod("Details", s.1, table(None, s.2.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())));
    }
    Some(Answer::new("organism", pods))
}

/// Ships: (names, title, &[(property, value)]).
const SHIPS: &[(&[&str], &str, &[(&str, &str)])] = &[
    (&["major islands in the indian ocean", "islands in the indian ocean", "indian ocean islands", "largest islands in the indian ocean", "major indian ocean islands"], "Major islands of the Indian Ocean (by area)", &[("Madagascar", "587,041 km² — Madagascar"), ("Sri Lanka", "65,610 km² — Sri Lanka"), ("Grande Terre (Kerguelen)", "6,675 km² — French Southern and Antarctic Lands"), ("Andaman Islands", "about 6,400 km² (archipelago) — India"), ("Socotra", "3,796 km² — Yemen"), ("Réunion", "2,512 km² — France"), ("Mauritius", "1,865 km² — Mauritius"), ("Unguja (Zanzibar)", "1,666 km² — Tanzania"), ("Nicobar Islands", "about 1,840 km² (archipelago) — India"), ("Grande Comore", "1,148 km² — Comoros"), ("Maldives", "298 km² of land in about 1,190 coral islands — Maldives"), ("Mahé", "157 km² — Seychelles")]),
    (&["hms archer", "hms archer (d78)"], "HMS Archer (D78)", &[("type", "escort aircraft carrier (Archer class), Royal Navy"), ("origin", "converted in the United States from the merchant ship Mormacland"), ("commissioned", "November 1941"), ("service", "Atlantic convoy escort in the Second World War")]),
    (&["uss bunker hill", "uss bunker hill (cv-17)", "uss bunker hill (cg-52)"], "USS Bunker Hill", &[("CV-17", "Essex-class aircraft carrier, commissioned May 25, 1943; badly damaged by two kamikaze hits off Okinawa on May 11, 1945"), ("CG-52", "Ticonderoga-class guided-missile cruiser, commissioned 1986, the first with the Mark 41 vertical launch system; decommissioned 2023"), ("named after", "the Battle of Bunker Hill (1775)")]),
    (&["hms atheling", "hms atheling (d51)"], "HMS Atheling (D51)", &[("type", "Ruler-class escort aircraft carrier, Royal Navy"), ("built", "in the United States, transferred under Lend-Lease"), ("commissioned", "1943"), ("service", "the Second World War, including the Indian Ocean; returned to the US Navy in 1946")]),
    (&["uss edisto", "uss edisto (agb-2)"], "USS Edisto (AGB-2)", &[("type", "Wind-class icebreaker"), ("commissioned", "1947 (US Navy)"), ("later", "transferred to the US Coast Guard in 1965 as USCGC Edisto (WAGB-284); decommissioned 1974"), ("service", "Arctic and Antarctic operations, including Operation Deep Freeze")]),
    (&["uss theodore roosevelt", "uss theodore roosevelt (cvn-71)", "cvn-71"], "USS Theodore Roosevelt (CVN-71)", &[("type", "Nimitz-class nuclear-powered aircraft carrier"), ("commissioned", "October 25, 1986"), ("displacement", "about 100,000 long tons"), ("length", "332.8 m (1,092 ft)"), ("homeport", "San Diego, California"), ("nickname", "\"Big Stick\"")]),
    (&["two brothers", "two brothers (ship)", "two brothers whaleship"], "Two Brothers (whaleship)", &[("type", "Nantucket whaling ship"), ("captain", "George Pollard Jr., formerly captain of the Essex"), ("wrecked", "February 1823 at French Frigate Shoals, Northwestern Hawaiian Islands"), ("wreck found", "2008; identified in 2011")]),
    (&["greenwich observatory", "royal observatory greenwich", "royal observatory, greenwich", "the royal observatory"], "Royal Observatory, Greenwich", &[("type", "astronomical observatory (now a museum)"), ("founded", "1675, by King Charles II"), ("first Astronomer Royal", "John Flamsteed"), ("location", "Greenwich Park, London (51.4769° N, 0.0005° W)"), ("significance", "defines the prime meridian (longitude 0°, 1884) through the Airy transit circle; Greenwich Mean Time")]),
    (&["comic-con international", "comiccon international", "comic con international", "san diego comic-con", "san diego comic con", "comic-con", "comiccon"], "Comic-Con International (San Diego Comic-Con)", &[("type", "comic book and popular-arts convention"), ("location", "San Diego Convention Center, San Diego, California"), ("first held", "1970"), ("held", "every July (four days plus a preview night)"), ("attendance", "about 135,000")]),
    (&["burning man", "burning man festival"], "Burning Man", &[("type", "annual gathering and arts event built around a temporary city"), ("location", "Black Rock City, Black Rock Desert, Nevada"), ("first held", "1986, on Baker Beach, San Francisco; in the Black Rock Desert since 1990"), ("held", "late August to Labor Day (nine days), ending with the burning of a large wooden effigy"), ("attendance", "about 70,000–80,000")]),
    (&["taste of chicago"], "Taste of Chicago", &[("type", "food festival"), ("location", "Grant Park, Chicago, Illinois"), ("first held", "July 4, 1980"), ("held", "every summer (now September); free admission, food and drink tickets sold"), ("note", "once billed as the world's largest food festival")]),
    (&["berlin hauptbahnhof", "berlin central station", "berlin hbf", "berlin main station"], "Berlin Hauptbahnhof", &[("type", "main railway station of Berlin, Germany"), ("opened", "May 26, 2006, on the site of the Lehrter Bahnhof"), ("layout", "a crossing station on two levels: east–west lines above (Stadtbahn) and north–south lines in a tunnel below"), ("note", "the largest crossing station in Europe")]),
    (&["us airways", "usairways", "u.s. airways"], "US Airways", &[("type", "major American airline (defunct)"), ("history", "founded in 1937 as All American Aviation; renamed Allegheny Airlines (1953), USAir (1979) and US Airways (1997); merged with America West in 2005"), ("end", "merged with American Airlines (2013); the last US Airways flight flew on October 17, 2015"), ("hubs", "Charlotte, Philadelphia, Phoenix and Washington National"), ("known for", "US Airways Flight 1549, landed on the Hudson River on January 15, 2009")]),
    (&["orange county california", "orange county, california", "orange county ca", "orange county, ca"], "Orange County, California", &[("population", "3,186,989 (2020 census)"), ("county seat", "Santa Ana"), ("largest city", "Anaheim"), ("area", "about 2,455 km² (948 sq mi) total"), ("formed", "1889, from part of Los Angeles County")]),
    (&["orange county florida", "orange county, florida", "orange county fl", "orange county, fl"], "Orange County, Florida", &[("population", "1,429,908 (2020 census)"), ("county seat", "Orlando"), ("largest city", "Orlando"), ("area", "about 2,600 km² (1,003 sq mi) total"), ("formed", "1824 as Mosquito County; renamed Orange County in 1845")]),
    (&["wbbm-tv", "wbbm tv"], "WBBM-TV", &[("type", "television station owned and operated by CBS"), ("market", "Chicago, Illinois"), ("channel", "virtual channel 2"), ("call letters", "shared with WBBM radio (780 AM)")]),
];

fn lists_and_ships(l: &str) -> Option<Answer> {
    if let Some((_, title, items)) = LISTS.iter().find(|x| x.0.contains(&l)) {
        return Some(Answer::new("list", vec![
            pod("Input", "Input interpretation", text(*title)),
            primary(pod("Result", "Result", text(items.iter().map(|x| x.0).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Details", table(None, items.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())),
        ]));
    }
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?(?:\s|$)|\bversus\b)\s*").split(l).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let v: Vec<&(&[&str], &str, &[(&str, &str)])> = parts.iter().map(|p| SHIPS.iter().find(|s| s.0.contains(&p.trim()))).collect::<Option<Vec<_>>>()?;
    if v.len() == 1 && l == "two brothers" {
        return None;
    }
    let mut pods = vec![
        pod("Input", "Input interpretation", text(v.iter().map(|s| s.1).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", "Result", text(v.iter().map(|s| format!("{}: {}", s.1, s.2[0].1)).collect::<Vec<_>>().join(" | ")))),
    ];
    for s in &v {
        pods.push(pod("Details", s.1, table(None, s.2.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())));
    }
    Some(Answer::new("ship", pods))
}

/// "AM 580", "FM 101.1": a broadcast frequency and its wavelength.
fn radio(l: &str) -> Option<Answer> {
    let m = re!(r"^(am|fm) (\d{2,4}(?:\.\d)?)(?: (?:radio|station|frequency))?$").captures(l)?;
    let v: f64 = m[2].parse().ok()?;
    let (hz, band, shown) = if &m[1] == "am" {
        if !(530.0..=1710.0).contains(&v) {
            return None;
        }
        (v * 1e3, "AM (medium wave) broadcast band, 530–1710 kHz in the Americas", format!("{v} kHz"))
    } else {
        if !(87.5..=108.0).contains(&v) {
            return None;
        }
        (v * 1e6, "FM (VHF) broadcast band, 87.5–108 MHz", format!("{v} MHz"))
    };
    let lambda = 299_792_458.0 / hz;
    Some(Answer::new("radio frequency", vec![
        pod("Input", "Input interpretation", text(format!("{} radio | {shown}", m[1].to_uppercase()))),
        primary(pod("Result", "Frequency", text(shown.clone()))),
        pod("Wavelength", "Wavelength", text(format!("{:.2} m", lambda))),
        pod("Band", "Band", text(band)),
        pod("Antenna", "Quarter-wave antenna length", text(format!("{:.2} m", lambda / 4.0))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = l.as_str();
    elevation(l).or_else(|| institutions(l)).or_else(|| creatures(l)).or_else(|| families(l)).or_else(|| poems(l)).or_else(|| history(l)).or_else(|| defs(l)).or_else(|| lists_and_ships(l)).or_else(|| radio(l)).or_else(|| living(l))
}
