//! Dinosaurs and other prehistoric animals: classification, age, size,
//! diet, where they lived and who named them, plus the main dinosaur
//! clades and comparisons.

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | classification (broad to narrow) | period | age (million years ago) |
/// length m | mass kg | diet | found in | named (year, by) | note
const DATA: &str = "\
tyrannosaurus rex,tyrannosaurus,t. rex,t rex,t-rex,trex|Dinosauria,Saurischia,Theropoda,Tyrannosauridae|Late Cretaceous|68–66|12.3|8400|carnivore|western North America|1905, Henry Fairfield Osborn|one of the last non-avian dinosaurs; bite force estimated at about 35,000–57,000 N
triceratops|Dinosauria,Ornithischia,Ceratopsia,Ceratopsidae|Late Cretaceous|68–66|9|9000|herbivore|western North America|1889, Othniel Charles Marsh|three horns and a large bony frill; lived alongside Tyrannosaurus
brachiosaurus|Dinosauria,Saurischia,Sauropodomorpha,Sauropoda,Brachiosauridae|Late Jurassic|154–153|22|40000|herbivore|western North America (Morrison Formation)|1903, Elmer S. Riggs|front legs longer than hind legs, holding the neck high; about 12 m tall
struthiomimus|Dinosauria,Saurischia,Theropoda,Ornithomimidae|Late Cretaceous|77–75|4.3|150|omnivore|Alberta, Canada|1917, Henry Fairfield Osborn|\"ostrich mimic\"; a fast, toothless runner
velociraptor|Dinosauria,Saurischia,Theropoda,Dromaeosauridae|Late Cretaceous|75–71|2|15|carnivore|Mongolia and China|1924, Henry Fairfield Osborn|feathered, turkey-sized; the \"fighting dinosaurs\" fossil shows one locked with a Protoceratops
stegosaurus|Dinosauria,Ornithischia,Thyreophora,Stegosauria|Late Jurassic|155–145|9|5000|herbivore|western North America and Portugal|1877, Othniel Charles Marsh|two rows of plates along the back and a spiked tail (thagomizer)
diplodocus|Dinosauria,Saurischia,Sauropodomorpha,Sauropoda,Diplodocidae|Late Jurassic|154–152|26|15000|herbivore|western North America|1878, Othniel Charles Marsh|very long whip-like tail
apatosaurus|Dinosauria,Saurischia,Sauropodomorpha,Sauropoda,Diplodocidae|Late Jurassic|152–151|22|20000|herbivore|western North America|1877, Othniel Charles Marsh|
brontosaurus|Dinosauria,Saurischia,Sauropodomorpha,Sauropoda,Diplodocidae|Late Jurassic|156–146|22|15000|herbivore|western North America|1879, Othniel Charles Marsh|long treated as a synonym of Apatosaurus; reinstated as a separate genus in 2015
allosaurus|Dinosauria,Saurischia,Theropoda,Allosauridae|Late Jurassic|155–145|8.5|2000|carnivore|western North America and Portugal|1877, Othniel Charles Marsh|the top predator of the Morrison Formation
spinosaurus|Dinosauria,Saurischia,Theropoda,Spinosauridae|Late Cretaceous|99–93.5|14|7000|piscivore|North Africa|1915, Ernst Stromer|the longest known carnivorous dinosaur; a large sail on its back and a semi-aquatic lifestyle
giganotosaurus|Dinosauria,Saurischia,Theropoda,Carcharodontosauridae|Late Cretaceous|99.6–97|12.5|7000|carnivore|Patagonia, Argentina|1995, Rodolfo Coria and Leonardo Salgado|
argentinosaurus|Dinosauria,Saurischia,Sauropodomorpha,Sauropoda,Titanosauria|Late Cretaceous|96–92|35|70000|herbivore|Patagonia, Argentina|1993, José Bonaparte and Rodolfo Coria|among the largest land animals ever
patagotitan|Dinosauria,Saurischia,Sauropodomorpha,Sauropoda,Titanosauria|Early Cretaceous|102|37|57000|herbivore|Patagonia, Argentina|2017, José Carballido et al.|
ankylosaurus|Dinosauria,Ornithischia,Thyreophora,Ankylosauridae|Late Cretaceous|68–66|7|5000|herbivore|western North America|1908, Barnum Brown|armoured body and a bony tail club
parasaurolophus|Dinosauria,Ornithischia,Ornithopoda,Hadrosauridae|Late Cretaceous|76.5–73|9.5|2500|herbivore|western North America|1922, William Parks|a long hollow head crest, probably used to make sounds
edmontosaurus|Dinosauria,Ornithischia,Ornithopoda,Hadrosauridae|Late Cretaceous|73–66|12|6000|herbivore|western North America|1917, Lawrence Lambe|
maiasaura|Dinosauria,Ornithischia,Ornithopoda,Hadrosauridae|Late Cretaceous|76.7|9|4000|herbivore|Montana, United States|1979, Jack Horner and Robert Makela|\"good mother lizard\"; nesting grounds show parental care
iguanodon|Dinosauria,Ornithischia,Ornithopoda,Iguanodontidae|Early Cretaceous|126–122|10|3000|herbivore|Europe|1825, Gideon Mantell|the second dinosaur ever named; thumb spikes
pachycephalosaurus|Dinosauria,Ornithischia,Marginocephalia,Pachycephalosauria|Late Cretaceous|70–66|4.5|450|herbivore|western North America|1943, Barnum Brown and Erich Schlaikjer|a skull dome up to 25 cm thick
protoceratops|Dinosauria,Ornithischia,Ceratopsia,Protoceratopsidae|Late Cretaceous|75–71|2|80|herbivore|Mongolia and China|1923, Walter Granger and William K. Gregory|
styracosaurus|Dinosauria,Ornithischia,Ceratopsia,Ceratopsidae|Late Cretaceous|75.5–75|5.5|2700|herbivore|Alberta, Canada|1913, Lawrence Lambe|long spikes on the frill
compsognathus|Dinosauria,Saurischia,Theropoda,Compsognathidae|Late Jurassic|150|1|3|carnivore|Germany and France|1859, Johann Andreas Wagner|chicken-sized
archaeopteryx|Dinosauria,Saurischia,Theropoda,Avialae|Late Jurassic|150.8–148.5|0.5|1|carnivore|Bavaria, Germany|1861, Hermann von Meyer|feathered transitional form between dinosaurs and birds
dilophosaurus|Dinosauria,Saurischia,Theropoda,Dilophosauridae|Early Jurassic|193|7|400|carnivore|Arizona, United States|1970, Samuel Welles|a pair of thin crests on the skull
deinonychus|Dinosauria,Saurischia,Theropoda,Dromaeosauridae|Early Cretaceous|115–108|3.4|75|carnivore|western North America|1969, John Ostrom|its discovery sparked the dinosaur renaissance and the bird-dinosaur link
utahraptor|Dinosauria,Saurischia,Theropoda,Dromaeosauridae|Early Cretaceous|135|6|500|carnivore|Utah, United States|1993, James Kirkland et al.|the largest known dromaeosaur
gallimimus|Dinosauria,Saurischia,Theropoda,Ornithomimidae|Late Cretaceous|70|6|450|omnivore|Mongolia|1972, Halszka Osmólska et al.|
carnotaurus|Dinosauria,Saurischia,Theropoda,Abelisauridae|Late Cretaceous|72–69.9|8|1500|carnivore|Patagonia, Argentina|1985, José Bonaparte|two horns above the eyes and tiny arms
therizinosaurus|Dinosauria,Saurischia,Theropoda,Therizinosauridae|Late Cretaceous|70|10|5000|herbivore|Mongolia|1954, Evgeny Maleev|hand claws up to about 1 m long
oviraptor|Dinosauria,Saurischia,Theropoda,Oviraptoridae|Late Cretaceous|75|1.6|30|omnivore|Mongolia|1924, Henry Fairfield Osborn|\"egg thief\", though the eggs it was found on were its own
baryonyx|Dinosauria,Saurischia,Theropoda,Spinosauridae|Early Cretaceous|130–125|9|1500|piscivore|England|1986, Alan Charig and Angela Milner|fish scales found in its stomach
microraptor|Dinosauria,Saurischia,Theropoda,Dromaeosauridae|Early Cretaceous|120|0.8|1|carnivore|Liaoning, China|2000, Xu Xing et al.|four feathered wings
plateosaurus|Dinosauria,Saurischia,Sauropodomorpha,Plateosauridae|Late Triassic|214–204|8|4000|herbivore|Europe|1837, Hermann von Meyer|
coelophysis|Dinosauria,Saurischia,Theropoda,Coelophysidae|Late Triassic|215|3|20|carnivore|New Mexico, United States|1889, Edward Drinker Cope|hundreds of skeletons found together at Ghost Ranch
herrerasaurus|Dinosauria,Saurischia,Herrerasauridae|Late Triassic|231|4.5|300|carnivore|Argentina|1963, Osvaldo Reig|one of the earliest dinosaurs
eoraptor|Dinosauria,Saurischia,Sauropodomorpha|Late Triassic|231|1|10|omnivore|Argentina|1993, Paul Sereno et al.|one of the earliest dinosaurs
megalosaurus|Dinosauria,Saurischia,Theropoda,Megalosauridae|Middle Jurassic|166|6|1000|carnivore|England|1824, William Buckland|the first non-avian dinosaur genus to be named
pteranodon|Pterosauria,Pteranodontidae|Late Cretaceous|86–84.5|1.8|25|piscivore|western North America|1876, Othniel Charles Marsh|a flying pterosaur, not a dinosaur; wingspan up to about 6 m
quetzalcoatlus|Pterosauria,Azhdarchidae|Late Cretaceous|68–66|3|250|carnivore|Texas, United States|1975, Douglas Lawson|a pterosaur, not a dinosaur; wingspan about 10–11 m, among the largest flying animals ever
mosasaurus|Squamata,Mosasauridae|Late Cretaceous|82–66|13|10000|carnivore|oceans worldwide|1822, William Conybeare|a marine reptile related to lizards and snakes, not a dinosaur
plesiosaurus|Plesiosauria,Plesiosauridae|Early Jurassic|199–175|3.5|450|piscivore|England|1821, Henry De la Beche and William Conybeare|a marine reptile, not a dinosaur
dimetrodon|Synapsida,Sphenacodontidae|Early Permian|295–272|3.5|250|carnivore|North America and Germany|1878, Edward Drinker Cope|a sail-backed synapsid, more closely related to mammals than to dinosaurs
woolly mammoth,mammoth|Mammalia,Proboscidea,Elephantidae|Pleistocene|0.8–0.004|4|6000|herbivore|northern Eurasia and North America|1799, Johann Friedrich Blumenbach|the last population survived on Wrangel Island until about 4,000 years ago
smilodon,sabre-toothed cat,saber-toothed cat,saber tooth tiger,sabre-toothed tiger|Mammalia,Carnivora,Felidae,Machairodontinae|Pleistocene|2.5–0.01|1.9|300|carnivore|the Americas|1842, Peter Wilhelm Lund|canines up to about 28 cm long
megalodon,otodus megalodon|Chondrichthyes,Lamniformes,Otodontidae|Miocene–Pliocene|23–3.6|16|50000|carnivore|oceans worldwide|1835, Louis Agassiz|an extinct shark known mainly from teeth up to 18 cm
";

/// clade | description | named
const CLADES: &[(&str, &str, &str)] = &[
    ("dinosauria", "dinosaurs: the last common ancestor of Triceratops and birds and all its descendants; appeared about 243–233 million years ago; non-avian dinosaurs died out 66 million years ago, birds survive", "1842, Richard Owen"),
    ("saurischia", "\"lizard-hipped\" dinosaurs: theropods (including birds) and sauropodomorphs", "1888, Harry Govier Seeley"),
    ("ornithischia", "\"bird-hipped\" dinosaurs: mostly beaked herbivores such as ceratopsians, ornithopods, stegosaurs and ankylosaurs", "1888, Harry Govier Seeley"),
    ("theropoda", "bipedal, mostly carnivorous dinosaurs; includes birds", "1881, Othniel Charles Marsh"),
    ("sauropodomorpha", "long-necked herbivorous dinosaurs and their early relatives", "1932, Friedrich von Huene"),
    ("sauropoda", "giant long-necked, four-legged herbivores, the largest land animals ever", "1878, Othniel Charles Marsh"),
    ("ornithopoda", "beaked herbivores, including the duck-billed hadrosaurs", "1881, Othniel Charles Marsh"),
    ("ceratopsia", "horned and frilled herbivores", "1890, Othniel Charles Marsh"),
    ("thyreophora", "armoured dinosaurs: stegosaurs and ankylosaurs", "1915, Franz Nopcsa"),
    ("hadrosauridae", "duck-billed dinosaurs", "1869, Edward Drinker Cope"),
    ("dromaeosauridae", "feathered \"raptors\" with an enlarged sickle claw on each foot", "1922, William Diller Matthew and Barnum Brown"),
    ("tyrannosauridae", "large-headed, short-armed predators of the Late Cretaceous", "1906, Henry Fairfield Osborn"),
    ("pterosauria", "flying reptiles — close relatives of dinosaurs, but not dinosaurs", "1834, Johann Jakob Kaup"),
];

fn rows() -> &'static Vec<Vec<&'static str>> {
    static R: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    R.get_or_init(|| DATA.lines().map(|l| l.split('|').collect()).collect())
}

fn find(name: &str) -> Option<&'static Vec<&'static str>> {
    let name = name.trim().trim_start_matches("the ").trim_end_matches(" dinosaur");
    rows().iter().find(|r| r[0].split(',').any(|n| n == name))
}

fn title(r: &[&str]) -> String {
    let n = r[0].split(',').next().unwrap_or("");
    let mut c = n.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn fmt_mass(kg: f64) -> String {
    if kg >= 1000.0 {
        format!("{} t ({} short tons)", crate::print::fmt_float(kg / 1000.0, 3), crate::print::fmt_float((kg / 907.185 * 10.0).round() / 10.0, 4))
    } else {
        format!("{} kg ({} lb)", crate::print::fmt_float(kg, 4), crate::print::fmt_float((kg * 2.20462).round(), 5))
    }
}

fn fmt_len(m: f64) -> String {
    format!("{} m ({} ft)", crate::print::fmt_float(m, 4), crate::print::fmt_float((m * 3.28084).round(), 4))
}

fn is_dino(r: &[&str]) -> bool {
    r[1].starts_with("Dinosauria")
}

fn card(r: &[&str]) -> Vec<Vec<Content>> {
    let mut v = vec![
        vec![text("classification"), text(r[1].replace(',', " › "))],
        vec![text("period"), text(r[2])],
        vec![text("lived"), text(format!("{} million years ago", r[3]))],
        vec![text("length"), text(fmt_len(r[4].parse().unwrap_or(0.0)))],
        vec![text("mass"), text(fmt_mass(r[5].parse().unwrap_or(0.0)))],
        vec![text("diet"), text(r[6])],
        vec![text("fossils found in"), text(r[7])],
        vec![text("named"), text(r[8])],
    ];
    if !r[9].is_empty() {
        v.push(vec![text("note"), text(r[9])]);
    }
    v
}

fn summary(r: &[&str]) -> String {
    let adj = match r[6] { "carnivore" => "carnivorous", "herbivore" => "herbivorous", "omnivore" => "omnivorous", "piscivore" => "fish-eating", x => x };
    let art = if adj.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    let kind = if is_dino(r) { "dinosaur" } else { "prehistoric animal" };
    format!("{art} {adj} {kind} ({}) from the {}, {} million years ago; about {} long and {}", r[1].rsplit(',').next().unwrap_or(""), r[2], r[3], fmt_len(r[4].parse().unwrap_or(0.0)), fmt_mass(r[5].parse().unwrap_or(0.0)))
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).to_string();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what is |what are |what was |who was |tell me about |show )?(?:the |a |an )?").replace(&l, "").to_string();

    // clades: "saurischia", "theropods"
    let clade_key = l.trim_end_matches(" dinosaurs").trim_end_matches('s');
    let clade_alias = match clade_key { "theropod" => "theropoda", "sauropod" => "sauropoda", "ornithopod" => "ornithopoda", "ceratopsian" => "ceratopsia", "hadrosaur" | "duck-billed dinosaur" => "hadrosauridae", "dromaeosaur" => "dromaeosauridae", "tyrannosaur" => "tyrannosauridae", "pterosaur" => "pterosauria", "dinosaur" | "dinosauria" | "list of dinosaur" | "dinosaur specie" | "dinosaur type" | "type of dinosaur" | "kinds of dinosaur" | "kind of dinosaur" => "dinosauria", x => x };
    if let Some((c, desc, named)) = CLADES.iter().find(|c| c.0 == clade_alias || c.0 == l) {
        let members: Vec<String> = rows().iter().filter(|r| r[1].to_lowercase().split(',').any(|x| x == *c)).map(|r| title(r)).collect();
        let mut c2 = c.chars();
        let cname = c2.next().map(|f| f.to_uppercase().collect::<String>() + c2.as_str()).unwrap_or_default();
        return Some(Answer::new("clade", vec![
            input(format!("{cname} | clade")),
            primary(pod("Result", "Description", text(*desc))),
            pod("Named", "Named", text(*named)),
            pod("Members", "Examples", text(members.join(", "))),
        ]));
    }

    // superlatives: "largest dinosaur", "smallest dinosaur", "heaviest dinosaur", "longest dinosaur"
    if let Some(m) = re!(r"^(largest|biggest|heaviest|longest|smallest|lightest|oldest|earliest|fastest|tallest) (?:known )?(carnivorous |herbivorous |predatory |flying )?(?:dinosaur|dino)s?(?: ever| of all time)?$").captures(&l) {
        let filt = |r: &&Vec<&str>| is_dino(r) && match m.get(2).map(|x| x.as_str().trim()) {
            Some("carnivorous" | "predatory") => matches!(r[6], "carnivore" | "piscivore"),
            Some("herbivorous") => r[6] == "herbivore",
            _ => true,
        };
        let f = |r: &Vec<&str>, i: usize| r[i].parse::<f64>().unwrap_or(0.0);
        let age = |r: &Vec<&str>| r[3].split('–').next().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
        let it = rows().iter().filter(filt);
        let (r, what) = match &m[1] {
            "largest" | "biggest" | "heaviest" => (it.max_by(|a, b| f(a, 5).total_cmp(&f(b, 5)))?, "mass"),
            "longest" | "tallest" => (it.max_by(|a, b| f(a, 4).total_cmp(&f(b, 4)))?, "length"),
            "smallest" | "lightest" => (it.min_by(|a, b| f(a, 5).total_cmp(&f(b, 5)))?, "mass"),
            "oldest" | "earliest" => (it.max_by(|a, b| age(a).total_cmp(&age(b)))?, "age"),
            _ => return None,
        };
        let val = match what { "mass" => fmt_mass(f(r, 5)), "length" => fmt_len(f(r, 4)), _ => format!("{} million years ago", r[3]) };
        return Some(Answer::new("dinosaur", vec![
            input(format!("{} {}dinosaur", &m[1], m.get(2).map(|x| x.as_str()).unwrap_or(""))),
            primary(pod("Result", "Result", text(format!("{} ({val})", title(r))))),
            pod("Properties", &title(r), table(None, card(r))),
            pod("Note", "Note", text("estimates for extinct animals vary widely between studies; among well-known genera only")),
        ]));
    }

    // "dinosaurs of the jurassic", "cretaceous dinosaurs"
    if let Some(m) = re!(r"^(?:dinosaurs (?:of|from|in) (?:the )?(triassic|jurassic|cretaceous)(?: period)?|(triassic|jurassic|cretaceous)(?: period)? dinosaurs)$").captures(&l) {
        let p = m.get(1).or(m.get(2))?.as_str();
        let list: Vec<String> = rows().iter().filter(|r| is_dino(r) && r[2].to_lowercase().contains(p)).map(|r| format!("{} ({})", title(r), r[2])).collect();
        return Some(Answer::new("dinosaurs", vec![input(format!("dinosaurs | {p} period")), primary(pod("Result", "Dinosaurs", text(list.join(" | "))))]));
    }

    // properties: "how long was a triceratops", "mass of brachiosaurus", "when did stegosaurus live"
    let prop = re!(r"^(?:(?:how (long|big|heavy|large|tall) (?:was|were|is|are) (?:an? |the )?(.+?))|(length|size|mass|weight|period|age|diet|discovery|classification) (?:of )?(?:an? |the )?(.+)|when did (?:an? |the )?(.+?) live|what did (?:an? |the )?(.+?) eat|(.+?) (length|size|mass|weight|period|diet))$").captures(&l);
    if let Some(m) = prop {
        let (p, name) = if let Some(p) = m.get(1) { (p.as_str(), &m[2]) } else if let Some(p) = m.get(3) { (p.as_str(), &m[4]) } else if let Some(n) = m.get(5) { ("period", n.as_str()) } else if let Some(n) = m.get(6) { ("diet", n.as_str()) } else { (m.get(8)?.as_str(), &m[7]) };
        let names: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+and\s+)\s*").split(name).collect();
        let items: Option<Vec<_>> = names.iter().map(|n| find(n)).collect();
        if let Some(items) = items {
            let val = |r: &Vec<&str>| match p {
                "long" | "length" | "tall" => fmt_len(r[4].parse().unwrap_or(0.0)),
                "heavy" | "mass" | "weight" => fmt_mass(r[5].parse().unwrap_or(0.0)),
                "big" | "large" | "size" => format!("{} long, {}", fmt_len(r[4].parse().unwrap_or(0.0)), fmt_mass(r[5].parse().unwrap_or(0.0))),
                "period" | "age" => format!("{}, {} million years ago", r[2], r[3]),
                "diet" => r[6].to_string(),
                "discovery" => r[8].to_string(),
                _ => r[1].replace(',', " › "),
            };
            let main = if items.len() == 1 { val(items[0]) } else { items.iter().map(|r| format!("{}: {}", title(r), val(r))).collect::<Vec<_>>().join(" | ") };
            return Some(Answer::new("dinosaur", vec![
                input(format!("{} | {p}", items.iter().map(|r| title(r)).collect::<Vec<_>>().join(", "))),
                primary(pod("Result", "Result", text(main))),
            ]));
        }
    }

    // cards and comparisons: "triceratops", "brachiosaurus, struthiomimus", "t rex vs spinosaurus"
    let l = l.trim_start_matches("compare ");
    let names: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+|\s+and\s+)\s*").split(l).collect();
    let items: Vec<_> = names.iter().map(|n| find(n)).collect::<Option<Vec<_>>>()?;
    if items.len() == 1 {
        let r = items[0];
        return Some(Answer::new("dinosaur", vec![
            input(format!("{} | {}", title(r), if is_dino(r) { "dinosaur" } else { "prehistoric animal" })),
            primary(pod("Result", "Summary", text(summary(r)))),
            pod("Properties", "Properties", table(None, card(r))),
        ]));
    }
    let mut header = vec![""];
    let tnames: Vec<String> = items.iter().map(|r| title(r)).collect();
    header.extend(tnames.iter().map(|s| s.as_str()));
    let cards: Vec<Vec<Vec<Content>>> = items.iter().map(|r| card(r)).collect();
    let labels = ["classification", "period", "lived", "length", "mass", "diet", "fossils found in", "named"];
    let t: Vec<Vec<Content>> = labels.iter().enumerate().map(|(i, lab)| {
        let mut row = vec![text(*lab)];
        for c in &cards {
            row.push(c[i][1].clone());
        }
        row
    }).collect();
    let big = items.iter().max_by(|a, b| a[5].parse::<f64>().unwrap_or(0.0).total_cmp(&b[5].parse::<f64>().unwrap_or(0.0)))?;
    let small = items.iter().min_by(|a, b| a[5].parse::<f64>().unwrap_or(0.0).total_cmp(&b[5].parse::<f64>().unwrap_or(0.0)))?;
    let ratio = big[5].parse::<f64>().unwrap_or(0.0) / small[5].parse::<f64>().unwrap_or(1.0);
    Some(Answer::new("dinosaurs", vec![
        input(format!("{} | comparison", tnames.join(", "))),
        primary(pod("Result", "Comparison", table(Some(header), t))),
        pod("Size", "Size", text(format!("{} was about {}× as heavy as {}", title(big), crate::print::fmt_float((ratio * 10.0).round() / 10.0, 4), title(small)))),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        for r in rows() {
            assert_eq!(r.len(), 10, "{r:?}");
            assert!(r[4].parse::<f64>().is_ok() && r[5].parse::<f64>().is_ok(), "{r:?}");
        }
    }
}
