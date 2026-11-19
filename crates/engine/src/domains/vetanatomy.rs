//! Skeletal anatomy of the dog and the horse: bones, joints, what they
//! connect to and the homologous human structures.
//!
//! "dog femur", "horse long pastern", "horse hock joint", "what does the
//! dog tibia connect to", "how many bones are in the dog's left hindfoot".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// animal ("dog", "horse" or "both") | names | kind | region | connects to |
/// human homologue | description
const PARTS: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
    ("both", "atlas;first cervical vertebra;c1", "bone", "neck (cervical vertebra C1)", "occipital condyles of the skull (atlanto-occipital joint, nodding) and the axis (atlantoaxial joint, rotation)", "atlas (C1)", "ring-shaped first neck vertebra with broad wings that can be felt behind the skull"),
    ("both", "axis;second cervical vertebra;c2", "bone", "neck (cervical vertebra C2)", "the atlas, by the dens (atlantoaxial joint), and the third cervical vertebra", "axis (C2)", "second neck vertebra with a long spinous process and the peg-like dens"),
    ("both", "scapula;shoulder blade", "bone", "thoracic limb (shoulder)", "the humerus (shoulder joint); attached to the trunk only by muscles, as there is no functional clavicle", "scapula", "flat triangular bone with a prominent spine; dogs have at most a vestigial clavicle and horses none"),
    ("both", "humerus;arm bone", "bone", "thoracic limb (arm)", "the scapula at the shoulder joint and the radius and ulna at the elbow joint", "humerus", "long bone of the upper foreleg"),
    ("dog", "radius", "bone", "thoracic limb (forearm)", "the humerus at the elbow and the carpal bones at the carpus; the ulna along its length", "radius", "the main weight-bearing forearm bone; dogs keep a separate, mobile ulna"),
    ("horse", "radius", "bone", "thoracic limb (forearm)", "the humerus at the elbow and the carpal bones at the carpus (the horse's 'knee'); fused with the reduced ulna", "radius", "the forearm bone; in the horse the ulna is reduced and fused to it"),
    ("both", "ulna", "bone", "thoracic limb (forearm)", "the humerus at the elbow (its olecranon forms the point of the elbow) and the radius", "ulna", "complete in the dog; reduced and fused to the radius in the horse"),
    ("dog", "carpus;carpal bones;wrist", "joint", "thoracic limb", "radius and ulna above, metacarpal bones below", "wrist", "seven carpal bones in two rows"),
    ("horse", "carpus;carpal bones;knee;knee joint", "joint", "thoracic limb", "the radius above and the cannon bone (third metacarpal) and splint bones below", "wrist", "the horse's 'knee' is homologous to the human wrist; seven or eight carpal bones in two rows"),
    ("horse", "cannon bone;third metacarpal;metacarpal iii;cannon", "bone", "thoracic limb (metacarpus)", "the carpus above and the long pastern at the fetlock joint", "third metacarpal (middle finger)", "the single large weight-bearing metacarpal; the second and fourth metacarpals are the slender splint bones"),
    ("horse", "splint bones;splint bone", "bone", "limb (metacarpus or metatarsus)", "the carpus or tarsus above; they taper and end without reaching the fetlock", "second and fourth metacarpals or metatarsals", "vestiges of the second and fourth digits"),
    ("horse", "long pastern;long pastern bone;proximal phalanx;first phalanx;p1", "bone", "digit III (pastern)", "the cannon bone at the fetlock joint and the short pastern at the pastern joint", "proximal phalanx of the middle finger or toe", "the first phalanx of the horse's single digit"),
    ("horse", "short pastern;short pastern bone;middle phalanx;second phalanx;p2", "bone", "digit III (pastern)", "the long pastern at the pastern joint and the coffin bone and navicular bone at the coffin joint", "middle phalanx of the middle finger or toe", "the second phalanx, mostly inside the hoof capsule's upper part"),
    ("horse", "coffin bone;pedal bone;distal phalanx;third phalanx;p3", "bone", "digit III (inside the hoof)", "the short pastern and the navicular bone at the coffin joint", "distal phalanx of the middle finger or toe", "the crescent-shaped bone inside the hoof"),
    ("horse", "navicular bone;distal sesamoid", "bone", "digit III (inside the hoof)", "the short pastern and the coffin bone", "a sesamoid bone (no exact human counterpart)", "small boat-shaped sesamoid behind the coffin joint; site of navicular disease"),
    ("horse", "fetlock;fetlock joint", "joint", "limb (between cannon and pastern)", "the cannon bone, the long pastern and the two proximal sesamoid bones", "knuckle (metacarpophalangeal or metatarsophalangeal joint)", "a hinge joint that takes great strain at speed"),
    ("horse", "pastern joint", "joint", "digit III", "the long and short pastern bones", "proximal interphalangeal joint", "a low-motion saddle joint"),
    ("horse", "coffin joint", "joint", "digit III (inside the hoof)", "the short pastern, coffin bone and navicular bone", "distal interphalangeal joint", "the joint within the hoof"),
    ("both", "pelvis;hip bone;os coxae", "bone", "pelvic limb (pelvic girdle)", "the sacrum (sacroiliac joint) and the femur (hip joint)", "pelvis (hip bone)", "ilium, ischium and pubis fused into each hip bone"),
    ("both", "femur;thigh bone", "bone", "pelvic limb (thigh)", "the acetabulum of the pelvis at the hip (coxofemoral) joint and the tibia and patella at the stifle joint", "femur", "long bone of the thigh; the strongest bone of the hindlimb"),
    ("both", "patella;kneecap", "bone", "pelvic limb (stifle)", "the trochlea of the femur, within the quadriceps tendon", "patella (kneecap)", "sesamoid bone; the horse's patella can lock in place so it can sleep standing"),
    ("both", "stifle;stifle joint", "joint", "pelvic limb", "the femur, tibia and patella, with menisci and cruciate ligaments", "knee", "the true knee of the hindlimb (in dogs, cranial cruciate ligament rupture is a common injury)"),
    ("both", "hip joint;coxofemoral joint;hip", "joint", "pelvic limb", "the head of the femur and the acetabulum of the pelvis", "hip joint", "ball-and-socket joint; hip dysplasia is common in large dog breeds"),
    ("both", "tibia;shin bone", "bone", "pelvic limb (crus or gaskin)", "the femur at the stifle joint, the talus at the hock (tarsocrural) joint and the fibula", "tibia", "the main bone of the lower hindleg"),
    ("both", "fibula", "bone", "pelvic limb (crus)", "the tibia, and in the dog the tarsus", "fibula", "thin and complete in the dog; reduced to a short splinter fused to the tibia in the horse"),
    ("dog", "tarsus;hock;hock joint;tarsal bones;ankle", "joint", "pelvic limb", "the tibia and fibula above and the metatarsal bones below", "ankle", "seven tarsal bones; the calcaneus forms the point of the hock"),
    ("horse", "tarsus;hock;hock joint;tarsal bones;ankle", "joint", "pelvic limb", "the tibia above and the cannon bone (third metatarsal) and splint bones below", "ankle", "six or seven tarsal bones; the calcaneal tuber forms the point of the hock; the tarsocrural joint does most of the movement"),
    ("dog", "metacarpals;metacarpal bones;metacarpus", "bone", "thoracic limb (forepaw)", "the carpus and the proximal phalanges", "metacarpals", "five metacarpals; the first carries the dewclaw"),
    ("dog", "metatarsals;metatarsal bones;metatarsus", "bone", "pelvic limb (hindpaw)", "the tarsus and the proximal phalanges", "metatarsals", "four main metatarsals (II–V); the first is small or absent"),
    ("both", "skull;cranium", "bone", "head", "the atlas at the atlanto-occipital joint; the mandible at the temporomandibular joints", "skull", "dog skulls vary greatly by breed (brachycephalic to dolichocephalic); the horse skull is long with a large diastema"),
    ("both", "mandible;lower jaw", "bone", "head", "the temporal bones of the skull (temporomandibular joints)", "mandible", "the lower jaw, carrying the lower teeth"),
    ("both", "sternum;breastbone", "bone", "thorax", "the costal cartilages of the sternal ribs", "sternum", "eight sternebrae in the dog, seven in the horse"),
];

const SHOULDER: &str = "scapula and humerus (meeting at the shoulder joint)";

fn animal(l: &str) -> Option<&'static str> {
    if re!(r"\b(?:dogs?|canine|canines|puppy|puppies)\b").is_match(l) {
        Some("dog")
    } else if re!(r"\b(?:horses?|equine|equines|pony|ponies)\b").is_match(l) {
        Some("horse")
    } else {
        None
    }
}

/// Organ systems and limb muscles: "digestive system of dogs",
/// "organs of the female dog body", "muscles of the left forelimb in horses".
fn systems(a: &str, rest: &str) -> Option<Answer> {
    let r = re!(r"^(?:what are |list |show |name )?(?:all )?").replace(rest, "").trim().to_string();
    let (title, rows): (String, Vec<(&str, &str)>) = if re!(r"^(?:digestive (?:system|tract|organs|anatomy)|gastrointestinal (?:system|tract)|gi tract|alimentary (?:canal|tract))$").is_match(&r) {
        let rows = if a == "dog" {
            vec![
                ("mouth", "42 permanent teeth (28 deciduous); carnassial teeth shear meat"),
                ("salivary glands", "parotid, mandibular, sublingual and zygomatic"),
                ("oesophagus", "carries food from the pharynx to the stomach"),
                ("stomach", "simple, single-chambered (monogastric)"),
                ("small intestine", "duodenum, jejunum and ileum"),
                ("accessory organs", "liver, gallbladder and pancreas"),
                ("large intestine", "caecum, colon (ascending, transverse, descending) and rectum"),
                ("anus", "flanked by the two anal sacs"),
            ]
        } else {
            vec![
                ("mouth", "36–44 permanent teeth with long-crowned (hypsodont) cheek teeth for grinding forage"),
                ("salivary glands", "parotid, mandibular and sublingual"),
                ("oesophagus", "a strong cardiac sphincter means horses can hardly vomit"),
                ("stomach", "small and simple (monogastric)"),
                ("small intestine", "duodenum, jejunum and ileum"),
                ("accessory organs", "liver and pancreas; horses have no gallbladder"),
                ("large intestine", "caecum, large (ascending) colon, small (descending) colon and rectum; fibre is fermented here (hindgut fermenter)"),
                ("anus", "end of the tract"),
            ]
        };
        (format!("{a} digestive system"), rows)
    } else if let Some(m) = re!(r"^(?:(female|male) )?(?:internal )?(?:organs|organ systems)(?: (female|male))?$").captures(&r) {
        let sex = m.get(1).or(m.get(2)).map(|x| x.as_str());
        let mut rows = vec![
            ("nervous system", "brain, spinal cord and nerves"),
            ("cardiovascular system", "heart (four chambers) and blood vessels; spleen"),
            ("respiratory system", "nasal cavity, larynx, trachea and lungs"),
            ("digestive system", if a == "dog" { "oesophagus, stomach, small and large intestine, liver, gallbladder, pancreas" } else { "oesophagus, stomach, small intestine, caecum and colon, liver, pancreas (no gallbladder)" }),
            ("urinary system", "kidneys, ureters, urinary bladder and urethra"),
            ("endocrine glands", "pituitary, thyroid, parathyroids, adrenals and pancreatic islets"),
            ("integument", "skin, hair and claws or hooves"),
        ];
        match (sex, a) {
            (Some("female"), "dog") => rows.push(("reproductive organs", "ovaries, uterus (a short body with two long horns), cervix, vagina and vulva; usually ten mammary glands in five pairs")),
            (Some("female"), _) => rows.push(("reproductive organs", "ovaries, uterus (a large body with two short horns), cervix, vagina and vulva; two mammary glands (teats) between the hind legs")),
            (Some("male"), "dog") => rows.push(("reproductive organs", "testes in the scrotum, epididymides, deferent ducts, the prostate (the dog's only accessory sex gland) and the penis with its bone (os penis)")),
            (Some("male"), _) => rows.push(("reproductive organs", "testes in the scrotum, epididymides, deferent ducts, accessory glands (ampullae, seminal vesicles, prostate, bulbourethral glands) and the penis")),
            _ => rows.push(("reproductive organs", "ovaries and uterus (female) or testes and prostate (male)")),
        }
        (format!("{}{a} organs", sex.map_or(String::new(), |s| format!("{s} "))), rows)
    } else if let Some(m) = re!(r"^(?:muscles (?:of )?(forelimb|front ?leg|foreleg|front limb|hindlimb|hind ?leg|back leg|hind limb)|(forelimb|front ?leg|foreleg|front limb|hindlimb|hind ?leg|back leg|hind limb) muscles)$").captures(&r) {
        let limb = m.get(1).or(m.get(2))?.as_str();
        let fore = limb.starts_with("fore") || limb.starts_with("front");
        let mut rows = if fore {
            vec![
                ("attaching the limb to the trunk", "trapezius, rhomboideus, brachiocephalicus, omotransversarius, latissimus dorsi, superficial and deep pectorals, serratus ventralis"),
                ("shoulder", "supraspinatus, infraspinatus, deltoideus, teres major, teres minor, subscapularis"),
                ("elbow", "biceps brachii and brachialis (flexors); triceps brachii, tensor fasciae antebrachii and anconeus (extensors)"),
                ("carpus and digits, extensors", "extensor carpi radialis, common digital extensor, lateral digital extensor, ulnaris lateralis"),
                ("carpus and digits, flexors", "flexor carpi radialis, flexor carpi ulnaris, superficial digital flexor, deep digital flexor"),
            ]
        } else {
            vec![
                ("hip and thigh", "gluteals (superficial, middle, deep), tensor fasciae latae, biceps femoris, semitendinosus, semimembranosus"),
                ("thigh, medial", "gracilis, sartorius, pectineus, adductors"),
                ("stifle", "quadriceps femoris (extensor); the hamstrings flex it"),
                ("hock", "gastrocnemius and superficial digital flexor (via the common calcaneal tendon)"),
                ("digits", "long and lateral digital extensors, deep digital flexor, cranial tibial and fibularis (peroneus) tertius"),
            ]
        };
        if a == "dog" && fore {
            rows.push(("forearm rotation", "pronator teres, pronator quadratus and supinator (dogs can rotate the forearm a little; horses cannot)"));
        }
        if a == "horse" {
            rows.push(("below the knee or hock", "no muscle bellies: only tendons and the suspensory ligament, which keeps the lower limb light"));
        }
        if a == "horse" && !fore {
            rows.push(("reciprocal apparatus", "fibularis tertius and the superficial digital flexor link stifle and hock so they flex and extend together"));
        }
        (format!("{a} {} muscles", if fore { "forelimb" } else { "hindlimb" }), rows)
    } else {
        return None;
    };
    let cap = |s: &str| { let mut c = s.chars(); c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str()) };
    Some(Answer::new(title.clone(), vec![
        pod("Input", "Input interpretation", text(title.clone())),
        primary(pod("Result", &cap(&title), table(None, rows.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect()))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let a = animal(&l)?;
    // drop the animal, sides and view words
    let rest = re!(r"\b(?:dogs?|canine|canines|horses?|equine|pony)(?:'s|s')?\b|\b(?:the|a|an|of|for|in|left|right|image|regional|location|body|view(?:ed)? from(?: the)? (?:left|right|front|back|above|below)|view)\b").replace_all(&l, " ").to_string();
    let rest = re!(r"\s+").replace_all(rest.trim(), " ").to_string();
    if let Some(x) = systems(a, &rest) {
        return Some(x);
    }
    // whole skeleton
    if re!(r"^(?:skeleton|bones|how many bones(?: are| does| do)?(?: have)?|number bones|skeletal system)$").is_match(&rest) {
        let (n, note) = if a == "dog" { ("about 320", "the count varies with tail length and dewclaws; humans have 206") } else { ("205", "about 205 in most horses (some breeds, such as Arabians, have fewer vertebrae); humans have 206") };
        return Some(Answer::new(format!("{a} skeleton"), vec![
            pod("Input", "Input interpretation", text(format!("{a} | skeleton"))),
            primary(pod("Result", "Number of bones", text(n))),
            pod("Note", "Note", text(note)),
        ]));
    }
    // "how many bones are in the dog's left hindfoot"
    if re!(r"^(?:how many )?bones (?:are )?(?:in )?(?:hind ?foot|hind ?paw|pes|back foot|hind foot)$").is_match(&rest) && a == "dog" {
        return Some(Answer::new("dog hindfoot bones", vec![
            pod("Input", "Input interpretation", text("dog | pes (hindfoot) | number of bones")),
            primary(pod("Result", "Number of bones", text("about 23: 7 tarsal bones, 4 metatarsals and 12 phalanges (up to 26 with a hind dewclaw)"))),
            pod("Table", "Bones", table(None, vec![
                vec![text("tarsal bones"), text("7 (talus, calcaneus, central, and first to fourth tarsal bones)")],
                vec![text("metatarsals"), text("4 (II–V); the first is usually small or absent")],
                vec![text("phalanges"), text("12: proximal, middle and distal phalanges of digits II–V")],
                vec![text("sesamoid bones"), text("not counted: paired sesamoids behind each metatarsophalangeal joint")],
            ])),
        ]));
    }
    if re!(r"^(?:what are )?(?:bones (?:of )?shoulder|shoulder bones|shoulder)$").is_match(&rest) {
        return Some(Answer::new(format!("{a} shoulder"), vec![
            pod("Input", "Input interpretation", text(format!("{a} | shoulder | bones"))),
            primary(pod("Result", "Bones", text(SHOULDER))),
            pod("Note", "Note", text("there is no functional clavicle, so the forelimb is attached to the trunk by muscles only (a synsarcosis)")),
        ]));
    }
    let (connect, homolog) = (rest.contains("connect") || rest.contains("articulat") || rest.contains("attach"), rest.contains("homolog") || rest.contains("human"));
    let key = re!(r"^(?:what (?:does|do) )?|\s*(?:connects?|articulates?|attach(?:es)?)(?: to| with)?$|\s*(?:homologous|homologue|homolog)(?: human)?(?: structures?| bones?)?$|\s*human (?:equivalent|homolog(?:ue)?)$").replace_all(&rest, "").trim().to_string();
    let p = PARTS.iter().find(|p| (p.0 == a || p.0 == "both") && p.1.split(';').any(|n| n == key || format!("{n} joint") == key || format!("{n} bone") == key))?;
    let name = p.1.split(';').next().unwrap();
    let (title, res) = if connect {
        ("Connections", p.4.to_string())
    } else if homolog {
        ("Homologous human structure", p.5.to_string())
    } else {
        ("Description", format!("{} ({})", p.6, p.3))
    };
    Some(Answer::new(format!("{a} {name}"), vec![
        pod("Input", "Input interpretation", text(format!("{a} | {name} ({})", p.2))),
        primary(pod("Result", title, text(res))),
        pod("Table", "Properties", table(None, vec![
            vec![text("type"), text(p.2)],
            vec![text("region"), text(p.3)],
            vec![text("connects to"), text(p.4)],
            vec![text("homologous human structure"), text(p.5)],
            vec![text("notes"), text(p.6)],
        ])),
    ]))
}
