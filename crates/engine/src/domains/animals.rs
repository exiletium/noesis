//! Animals: species facts (size, speed, lifespan, jumps, wingspans,
//! taxonomy, conservation status), higher taxa, and domestic breeds of
//! dogs, cats, cattle, horses, chickens, turkeys, pigs, sheep and goats,
//! which can be listed, filtered, ranked and compared.
//!
//! Figures are typical adult ranges from standard references (breed
//! standards, IUCN, natural-history handbooks); individuals vary widely.

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
    fmt_float(v, 4)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(String, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

type Range = Option<(f64, f64)>;

/// Parse "key=value;key=value" where a value may itself contain semicolons:
/// a piece without "=" continues the previous value.
pub(crate) fn key_values(s: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = vec![];
    for piece in s.split(';') {
        match piece.split_once('=') {
            Some((k, v)) if !k.contains(' ') || k.split_whitespace().count() <= 3 => out.push((k.trim().to_string(), v.trim().to_string())),
            _ => {
                if let Some(last) = out.last_mut() {
                    last.1.push_str(";");
                    last.1.push_str(piece);
                }
            }
        }
    }
    out
}

fn range(s: &str) -> Range {
    let s = s.trim();
    if s == "-" || s.is_empty() {
        return None;
    }
    match s.split_once('-') {
        Some((a, b)) => Some((a.trim().parse().ok()?, b.trim().parse().ok()?)),
        None => {
            let v: f64 = s.parse().ok()?;
            Some((v, v))
        }
    }
}

fn mid(r: Range) -> Option<f64> {
    r.map(|(a, b)| (a + b) / 2.0)
}

fn span(r: (f64, f64), unit: &str) -> String {
    if r.0 == r.1 { format!("{} {unit}", n(r.0)) } else { format!("{}–{} {unit}", n(r.0), n(r.1)) }
}

fn kg_text(r: (f64, f64)) -> String {
    if r.1 < 1.0 {
        return span((r.0 * 1000.0, r.1 * 1000.0), "g");
    }
    format!("{}  ({})", span(r, "kg"), span((r.0 * 2.204_62, r.1 * 2.204_62), "lb"))
}

fn m_text(r: (f64, f64)) -> String {
    if r.1 < 1.0 {
        return format!("{}  ({})", span((r.0 * 100.0, r.1 * 100.0), "cm"), span((r.0 * 39.3701, r.1 * 39.3701), "in"));
    }
    format!("{}  ({})", span(r, "m"), span((r.0 * 3.280_84, r.1 * 3.280_84), "ft"))
}

fn cm_text(r: (f64, f64)) -> String {
    format!("{}  ({})", span(r, "cm"), span((r.0 / 2.54, r.1 / 2.54), "in"))
}

fn kmh(v: f64) -> String {
    format!("{} km/h ({} mph)", n(v), n(v / 1.609_344))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// species
// ---------------------------------------------------------------------------

struct Species {
    names: Vec<String>,
    sci: String,
    class: String,
    order: String,
    family: String,
    mass: Range,
    length: Range,
    life: Range,
    speed: Option<f64>,
    status: String,
    range: String,
    extra: Vec<(String, String)>,
}

impl Species {
    fn name(&self) -> &str {
        &self.names[0]
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.extra.iter().find(|e| e.0 == k).map(|e| e.1.as_str())
    }
}

// names | scientific name | class | order | family | mass kg | length m | lifespan y | top speed km/h | status | range | extras
const SPECIES: &str = "\
jaguar|Panthera onca|Mammalia|Carnivora|Felidae|56-96|1.1-1.85|12-15|80|Near Threatened|Mexico to northern Argentina|jump=leaps several metres; the strongest bite of any big cat relative to size;diet=carnivore (capybaras, peccaries, caimans, deer)
tiger|Panthera tigris|Mammalia|Carnivora|Felidae|90-310|2.5-3.9|10-15|65|Endangered|South and Southeast Asia, Russian Far East|jump=about 6–10 m (20–33 ft) horizontally and up to about 5 m (16 ft) vertically;diet=carnivore (deer, wild boar)
lion|Panthera leo|Mammalia|Carnivora|Felidae|120-250|1.4-2.5|10-14|80|Vulnerable|sub-Saharan Africa; Gir Forest, India|jump=about 11 m (36 ft) horizontally;diet=carnivore (zebra, wildebeest, buffalo)
leopard|Panthera pardus|Mammalia|Carnivora|Felidae|30-90|0.9-1.9|12-17|58|Vulnerable|Africa and Asia|jump=about 6 m (20 ft) horizontally, 3 m (10 ft) vertically;diet=carnivore
snow leopard|Panthera uncia|Mammalia|Carnivora|Felidae|22-55|0.75-1.3|15-18|65|Vulnerable|mountains of Central and South Asia|jump=up to about 15 m (50 ft) horizontally;diet=carnivore (blue sheep, ibex)
cheetah|Acinonyx jubatus|Mammalia|Carnivora|Felidae|21-72|1.1-1.5|10-12|112|Vulnerable|Africa; a few in Iran|accel=0 to 96 km/h (60 mph) in about 3 seconds;diet=carnivore (gazelles, impala)
puma;cougar;mountain lion|Puma concolor|Mammalia|Carnivora|Felidae|29-100|1.0-1.6|8-13|80|Least Concern|Canada to southern Chile|jump=up to about 5.5 m (18 ft) vertically and 12 m (40 ft) horizontally;diet=carnivore (deer)
lynx;eurasian lynx|Lynx lynx|Mammalia|Carnivora|Felidae|18-30|0.8-1.3|15-17|80|Least Concern|Europe and Siberia|diet=carnivore (hares, roe deer)
bobcat|Lynx rufus|Mammalia|Carnivora|Felidae|6-14|0.5-1.0|7-10|48|Least Concern|southern Canada to Mexico|diet=carnivore
red fox|Vulpes vulpes|Mammalia|Carnivora|Canidae|2.2-14|0.45-0.9|2-5|50|Least Concern|Northern Hemisphere and Australia|jump=up to about 2 m (6.5 ft) high;diet=omnivore (rodents, rabbits, fruit)
arctic fox|Vulpes lagopus|Mammalia|Carnivora|Canidae|1.4-9|0.46-0.68|3-6|48|Least Concern|Arctic tundra|diet=omnivore
gray wolf;grey wolf;wolf|Canis lupus|Mammalia|Carnivora|Canidae|30-80|1.0-1.6|6-8|60|Least Concern|North America and Eurasia|diet=carnivore (deer, elk, moose)
coyote|Canis latrans|Mammalia|Carnivora|Canidae|7-21|0.75-1.0|10-14|65|Least Concern|North and Central America|diet=omnivore
spotted hyena;hyena|Crocuta crocuta|Mammalia|Carnivora|Hyaenidae|40-86|0.95-1.65|12-25|60|Least Concern|sub-Saharan Africa|diet=carnivore (hunts and scavenges)
brown bear;grizzly bear;grizzly|Ursus arctos|Mammalia|Carnivora|Ursidae|80-600|1.5-2.8|20-30|56|Least Concern|North America and Eurasia|diet=omnivore
polar bear|Ursus maritimus|Mammalia|Carnivora|Ursidae|150-700|1.8-3.0|25-30|40|Vulnerable|Arctic sea ice|swim=about 10 km/h;diet=carnivore (seals)
american black bear;black bear|Ursus americanus|Mammalia|Carnivora|Ursidae|40-300|1.2-2.0|18-25|56|Least Concern|North America|diet=omnivore
giant panda;panda|Ailuropoda melanoleuca|Mammalia|Carnivora|Ursidae|70-125|1.2-1.9|20-30|32|Vulnerable|mountains of central China|diet=bamboo (about 12–38 kg a day)
red panda|Ailurus fulgens|Mammalia|Carnivora|Ailuridae|3-6|0.5-0.64|8-10|-|Endangered|eastern Himalayas and southwestern China|diet=mostly bamboo
european badger;badger|Meles meles|Mammalia|Carnivora|Mustelidae|7-17|0.6-0.9|6-14|30|Least Concern|Europe and western Asia|diet=omnivore (earthworms, insects, fruit)
honey badger;ratel|Mellivora capensis|Mammalia|Carnivora|Mustelidae|6-16|0.55-0.77|7-24|30|Least Concern|Africa and southern Asia|diet=omnivore
sea otter|Enhydra lutris|Mammalia|Carnivora|Mustelidae|14-45|1.0-1.5|10-20|9|Endangered|North Pacific coasts|diet=sea urchins, crabs, clams
meerkat|Suricata suricatta|Mammalia|Carnivora|Herpestidae|0.62-0.97|0.25-0.35|12-14|32|Least Concern|southern Africa|diet=insects
raccoon|Procyon lotor|Mammalia|Carnivora|Procyonidae|2-14|0.4-0.7|2-3|24|Least Concern|North America|diet=omnivore
african elephant;african bush elephant;elephant|Loxodonta africana|Mammalia|Proboscidea|Elephantidae|2700-6000|5.5-7.5|60-70|40|Endangered|sub-Saharan Africa|height=3.0–4.0 m (10–13 ft) at the shoulder;diet=herbivore (about 150 kg of vegetation a day)
asian elephant|Elephas maximus|Mammalia|Proboscidea|Elephantidae|2000-5000|5.5-6.5|48-60|40|Endangered|South and Southeast Asia|height=2.5–3.2 m at the shoulder;diet=herbivore
giraffe|Giraffa camelopardalis|Mammalia|Artiodactyla|Giraffidae|700-1300|3.8-4.7|20-25|60|Vulnerable|savannas of sub-Saharan Africa|height=4.3–5.7 m (14–19 ft) to the top of the head;diet=herbivore (acacia leaves)
hippopotamus;hippo|Hippopotamus amphibius|Mammalia|Artiodactyla|Hippopotamidae|1500-3200|3.3-5.2|40-50|30|Vulnerable|sub-Saharan Africa|diet=herbivore (grass)
white rhinoceros;white rhino;rhinoceros;rhino|Ceratotherium simum|Mammalia|Perissodactyla|Rhinocerotidae|1800-3600|3.4-4.0|40-50|50|Near Threatened|southern and eastern Africa|diet=herbivore (grass)
plains zebra;zebra|Equus quagga|Mammalia|Perissodactyla|Equidae|175-385|2.2-2.5|20-25|65|Near Threatened|eastern and southern Africa|diet=herbivore
camel;dromedary|Camelus dromedarius|Mammalia|Artiodactyla|Camelidae|300-600|2.2-3.4|40-50|65|domesticated|North Africa, Middle East; feral in Australia|height=1.8–2.0 m at the hump;diet=herbivore
moose;elk (eurasia)|Alces alces|Mammalia|Artiodactyla|Cervidae|200-700|2.4-3.2|15-25|56|Least Concern|northern North America and Eurasia|height=1.4–2.1 m at the shoulder;diet=herbivore
white-tailed deer|Odocoileus virginianus|Mammalia|Artiodactyla|Cervidae|45-140|1.5-2.0|6-14|75|Least Concern|the Americas|jump=up to about 2.5 m (8 ft) high and 9 m (30 ft) long;diet=herbivore
pronghorn|Antilocapra americana|Mammalia|Artiodactyla|Antilocapridae|40-65|1.3-1.5|10-15|88|Least Concern|western North America|note=the fastest land animal over long distances;diet=herbivore
american bison;bison|Bison bison|Mammalia|Artiodactyla|Bovidae|400-1000|2.0-3.5|15-20|55|Near Threatened|North American plains|diet=herbivore (grass)
blue wildebeest;wildebeest|Connochaetes taurinus|Mammalia|Artiodactyla|Bovidae|118-290|1.7-2.4|20|80|Least Concern|eastern and southern Africa|diet=herbivore
thomson's gazelle;gazelle|Eudorcas thomsonii|Mammalia|Artiodactyla|Bovidae|15-35|0.8-1.2|10-15|80|Near Threatened|Kenya and Tanzania|diet=herbivore
red kangaroo;kangaroo|Osphranter rufus|Mammalia|Diprotodontia|Macropodidae|18-90|1.0-1.6|12-18|56|Least Concern|arid Australia|jump=up to about 8–9 m (26–30 ft) in one bound and about 3 m (10 ft) high;diet=herbivore
koala|Phascolarctos cinereus|Mammalia|Diprotodontia|Phascolarctidae|4-15|0.6-0.85|13-18|-|Vulnerable|eastern Australia|diet=eucalyptus leaves
platypus|Ornithorhynchus anatinus|Mammalia|Monotremata|Ornithorhynchidae|0.7-2.4|0.4-0.5|10-17|-|Near Threatened|eastern Australia and Tasmania|note=lays eggs; males have venomous spurs;diet=aquatic invertebrates
western gorilla;gorilla|Gorilla gorilla|Mammalia|Primates|Hominidae|70-200|1.25-1.8|35-40|40|Critically Endangered|central Africa|diet=mostly plants
chimpanzee;chimp|Pan troglodytes|Mammalia|Primates|Hominidae|40-60|0.63-0.94|40-50|40|Endangered|western and central Africa|diet=omnivore (fruit, leaves, insects, meat)
bornean orangutan;orangutan|Pongo pygmaeus|Mammalia|Primates|Hominidae|30-90|1.2-1.5|35-45|-|Critically Endangered|Borneo|diet=mostly fruit
three-toed sloth;sloth|Bradypus variegatus|Mammalia|Pilosa|Bradypodidae|2.2-6.3|0.42-0.8|20-30|0.24|Least Concern|Central and South America|diet=leaves
giant anteater;anteater|Myrmecophaga tridactyla|Mammalia|Pilosa|Myrmecophagidae|18-50|1.8-2.2|14-16|50|Vulnerable|Central and South America|diet=ants and termites (up to 35,000 a day)
nine-banded armadillo;armadillo|Dasypus novemcinctus|Mammalia|Cingulata|Dasypodidae|2.5-6.5|0.6-0.8|7-20|48|Least Concern|southern United States to Argentina|diet=insects
african pygmy hedgehog;four-toed hedgehog;atelerix albiventris|Atelerix albiventris|Mammalia|Eulipotyphla|Erinaceidae|0.25-0.6|0.15-0.25|3-5|-|Least Concern|central and eastern Africa|note=commonly kept as a pet; lives up to about 8 years in captivity;diet=insectivore
european hedgehog;hedgehog|Erinaceus europaeus|Mammalia|Eulipotyphla|Erinaceidae|0.4-1.2|0.2-0.3|2-5|-|Least Concern|western Europe|diet=insectivore
orinoco sword-nosed bat;lonchorhina orinocensis|Lonchorhina orinocensis|Mammalia|Chiroptera|Phyllostomidae|0.01-0.015|0.05-0.06|-|-|Vulnerable|Orinoco basin of Venezuela and Colombia|note=a leaf-nosed bat with a very long, sword-like nose leaf;diet=insectivore
little brown bat|Myotis lucifugus|Mammalia|Chiroptera|Vespertilionidae|0.0055-0.0125|0.06-0.1|6-7|35|Endangered|North America|wingspan=22–27 cm;diet=insects
capybara|Hydrochoerus hydrochaeris|Mammalia|Rodentia|Caviidae|35-66|1.06-1.34|8-10|35|Least Concern|South America|note=the largest living rodent;diet=herbivore
north american beaver;beaver|Castor canadensis|Mammalia|Rodentia|Castoridae|11-32|0.74-0.9|10-12|8|Least Concern|North America|diet=bark and aquatic plants
eastern gray squirrel;gray squirrel;squirrel|Sciurus carolinensis|Mammalia|Rodentia|Sciuridae|0.4-0.6|0.23-0.3|6-12|32|Least Concern|eastern North America; introduced in Britain|jump=up to about 3 m (10 ft) horizontally;diet=nuts and seeds
house mouse|Mus musculus|Mammalia|Rodentia|Muridae|0.012-0.03|0.075-0.1|1-2|13|Least Concern|worldwide|diet=omnivore
brown rat|Rattus norvegicus|Mammalia|Rodentia|Muridae|0.25-0.5|0.2-0.28|2-3|13|Least Concern|worldwide|diet=omnivore
european rabbit|Oryctolagus cuniculus|Mammalia|Lagomorpha|Leporidae|1.2-2.5|0.34-0.5|9|56|Endangered|native to Iberia; introduced worldwide|jump=about 1 m high and 3 m long;diet=herbivore
brown hare;hare|Lepus europaeus|Mammalia|Lagomorpha|Leporidae|2.5-7|0.5-0.75|4-12|70|Least Concern|Europe and western Asia|diet=herbivore
blue whale|Balaenoptera musculus|Mammalia|Artiodactyla (Cetacea)|Balaenopteridae|100000-150000|24-30|80-90|50|Endangered|all oceans|note=the largest animal known to have lived;diet=krill (up to 3.6 t a day)
humpback whale|Megaptera novaeangliae|Mammalia|Artiodactyla (Cetacea)|Balaenopteridae|25000-30000|12-16|45-50|27|Least Concern|all oceans|diet=krill and small fish
sperm whale|Physeter macrocephalus|Mammalia|Artiodactyla (Cetacea)|Physeteridae|14000-41000|11-16|60-70|35|Vulnerable|all oceans|note=dives to over 2,000 m;diet=squid
orca;killer whale|Orcinus orca|Mammalia|Artiodactyla (Cetacea)|Delphinidae|3600-6000|6-8|50-90|56|Data Deficient|all oceans|diet=fish, seals, other whales
bottlenose dolphin;dolphin|Tursiops truncatus|Mammalia|Artiodactyla (Cetacea)|Delphinidae|150-650|2-4|40-50|35|Least Concern|warm and temperate seas|diet=fish and squid
peregrine falcon|Falco peregrinus|Aves|Falconiformes|Falconidae|0.33-1.5|0.34-0.58|15|90|Least Concern|every continent except Antarctica|dive=up to about 390 km/h (240 mph) in a hunting stoop — the fastest animal;wingspan=0.74–1.2 m (2.4–3.9 ft);diet=birds
golden eagle|Aquila chrysaetos|Aves|Accipitriformes|Accipitridae|3-6.7|0.66-1.0|20-30|50|Least Concern|Northern Hemisphere|dive=about 240–320 km/h in a stoop;wingspan=1.8–2.34 m (5.9–7.7 ft);diet=carnivore
bald eagle|Haliaeetus leucocephalus|Aves|Accipitriformes|Accipitridae|3-6.3|0.7-1.0|20-30|56|Least Concern|North America|dive=about 120–160 km/h;wingspan=1.8–2.3 m (5.9–7.5 ft);diet=fish
greater flamingo;flamingo|Phoenicopterus roseus|Aves|Phoenicopteriformes|Phoenicopteridae|2-4|1.1-1.5|30-40|55|Least Concern|Africa, southern Europe, the Middle East and South Asia|wingspan=1.4–1.65 m (4.6–5.4 ft);height=1.1–1.5 m (3.6–4.9 ft);diet=brine shrimp and algae
ostrich|Struthio camelus|Aves|Struthioniformes|Struthionidae|63-145|1.7-2.0|40-45|70|Least Concern|Africa|height=2.1–2.8 m (6.9–9.2 ft);note=the largest living bird; lays the largest eggs;diet=omnivore
emu|Dromaius novaehollandiae|Aves|Casuariiformes|Dromaiidae|18-60|1.5-1.9|10-20|50|Least Concern|Australia|height=1.5–1.9 m;diet=omnivore
emperor penguin|Aptenodytes forsteri|Aves|Sphenisciformes|Spheniscidae|22-45|1.0-1.3|20|9|Near Threatened|Antarctica|swim=about 9 km/h, bursts to 14 km/h;diet=fish, krill, squid
wandering albatross;albatross|Diomedea exulans|Aves|Procellariiformes|Diomedeidae|6-12|1.07-1.35|50-60|-|Vulnerable|Southern Ocean|wingspan=2.5–3.5 m (8.2–11.5 ft) — the largest of any living bird;diet=squid and fish
ruby-throated hummingbird;hummingbird|Archilochus colubris|Aves|Apodiformes|Trochilidae|0.002-0.006|0.07-0.09|3-5|50|Least Concern|eastern North America; winters in Central America|wingspan=8–11 cm;note=beats its wings about 50 times a second;diet=nectar and small insects
great horned owl|Bubo virginianus|Aves|Strigiformes|Strigidae|0.9-2.5|0.43-0.64|13|65|Least Concern|the Americas|wingspan=1.0–1.6 m;diet=carnivore
snowy owl|Bubo scandiacus|Aves|Strigiformes|Strigidae|1.6-3|0.52-0.71|10|80|Vulnerable|Arctic|wingspan=1.25–1.5 m;diet=lemmings
american crow;crow|Corvus brachyrhynchos|Aves|Passeriformes|Corvidae|0.32-0.6|0.4-0.53|7-8|50|Least Concern|North America|wingspan=0.85–1.0 m;diet=omnivore
common ostrich;mute swan;swan|Cygnus olor|Aves|Anseriformes|Anatidae|9-14|1.25-1.7|20-30|88|Least Concern|Europe and Asia|wingspan=2.0–2.4 m;diet=aquatic plants
mallard|Anas platyrhynchos|Aves|Anseriformes|Anatidae|0.7-1.6|0.5-0.65|5-10|88|Least Concern|Northern Hemisphere|wingspan=0.81–0.98 m;diet=omnivore
common swift;swift|Apus apus|Aves|Apodiformes|Apodidae|0.036-0.05|0.16-0.17|9|110|Least Concern|Europe, Asia, Africa|wingspan=38–40 cm;note=stays airborne for up to 10 months at a time;diet=flying insects
saltwater crocodile;crocodile|Crocodylus porosus|Reptilia|Crocodilia|Crocodylidae|400-1000|4-6|70|29|Least Concern|South and Southeast Asia, northern Australia|swim=about 24–29 km/h in bursts;diet=carnivore
american alligator;alligator|Alligator mississippiensis|Reptilia|Crocodilia|Alligatoridae|200-450|3.4-4.5|35-50|18|Least Concern|southeastern United States|swim=about 32 km/h in bursts;diet=carnivore
komodo dragon|Varanus komodoensis|Reptilia|Squamata|Varanidae|70-90|2.6-3.0|30|20|Endangered|Komodo and nearby Indonesian islands|note=the largest living lizard;diet=carnivore
green sea turtle|Chelonia mydas|Reptilia|Testudines|Cheloniidae|68-190|0.78-1.12|60-80|35|Endangered|tropical and subtropical seas|diet=seagrass and algae (adults)
galapagos tortoise|Chelonoidis niger|Reptilia|Testudines|Testudinidae|150-417|1.5-1.8|100-150|0.3|Endangered|Galápagos Islands|diet=herbivore
king cobra|Ophiophagus hannah|Reptilia|Squamata|Elapidae|6-12|3.0-4.0|20|19|Vulnerable|South and Southeast Asia|note=the longest venomous snake, up to 5.85 m;diet=other snakes
black mamba|Dendroaspis polylepis|Reptilia|Squamata|Elapidae|1.6-2|2.0-3.0|11|19|Least Concern|sub-Saharan Africa|note=one of the fastest snakes;diet=small mammals and birds
green anaconda;anaconda|Eunectes murinus|Reptilia|Squamata|Boidae|30-250|4.5-7|10|8|Least Concern|tropical South America|note=the heaviest snake;diet=carnivore
reticulated python|Malayopython reticulatus|Reptilia|Squamata|Pythonidae|20-75|3-6.5|20-25|1.6|Least Concern|Southeast Asia|note=the longest snake;diet=carnivore
green iguana;iguana|Iguana iguana|Reptilia|Squamata|Iguanidae|4-8|1.2-1.7|10-15|35|Least Concern|Central and South America|diet=herbivore
american bullfrog;bullfrog|Lithobates catesbeianus|Amphibia|Anura|Ranidae|0.5-0.8|0.09-0.2|8-10|-|Least Concern|eastern North America|jump=up to about 2 m (6.5 ft) in one leap;diet=carnivore
axolotl|Ambystoma mexicanum|Amphibia|Urodela|Ambystomatidae|0.06-0.23|0.15-0.45|10-15|-|Critically Endangered|lakes near Mexico City|note=regenerates limbs and organs;diet=carnivore
great white shark;white shark|Carcharodon carcharias|Chondrichthyes|Lamniformes|Lamnidae|520-1100|3.4-6.4|70|56|Vulnerable|coastal waters of all major oceans|diet=fish, seals, sea lions
whale shark|Rhincodon typus|Chondrichthyes|Orectolobiformes|Rhincodontidae|15000-20000|10-12|80-130|5|Endangered|tropical oceans|note=the largest fish;diet=plankton
shortfin mako shark;mako shark|Isurus oxyrinchus|Chondrichthyes|Lamniformes|Lamnidae|60-570|2-4|30|70|Endangered|temperate and tropical seas|note=the fastest shark;diet=fish and squid
giant manta ray;manta ray|Mobula birostris|Chondrichthyes|Myliobatiformes|Mobulidae|1350-3000|4.5-7|40|24|Endangered|tropical and subtropical oceans|wingspan=up to about 7 m;diet=plankton
atlantic salmon|Salmo salar|Actinopterygii|Salmoniformes|Salmonidae|3.6-6|0.7-0.76|4-8|-|Least Concern|North Atlantic and its rivers|jump=up to about 3.7 m (12 ft) up waterfalls;diet=carnivore
sailfish|Istiophorus platypterus|Actinopterygii|Istiophoriformes|Istiophoridae|54-100|1.9-3.15|4-5|110|Least Concern|warm and temperate oceans|note=often cited as the fastest fish; measured sprints are closer to 30–40 km/h;diet=fish and squid
atlantic bluefin tuna;bluefin tuna|Thunnus thynnus|Actinopterygii|Scombriformes|Scombridae|225-680|2-3|15-30|70|Least Concern|Atlantic Ocean and Mediterranean|diet=fish and squid
clownfish;ocellaris clownfish|Amphiprion ocellaris|Actinopterygii|Ovalentaria|Pomacentridae|0.25-0.25|0.08-0.11|6-10|-|Least Concern|Indo-Pacific reefs|note=lives among sea anemones;diet=omnivore
goldfish|Carassius auratus|Actinopterygii|Cypriniformes|Cyprinidae|0.1-0.3|0.1-0.2|10-15|-|Least Concern|native to East Asia|diet=omnivore
western honey bee;honey bee;honeybee|Apis mellifera|Insecta|Hymenoptera|Apidae|0.0001-0.0001|0.012-0.015|0.1-0.15|24|Data Deficient|worldwide (domesticated)|note=workers live about 5–6 weeks in summer; queens 2–5 years;diet=nectar and pollen
monarch butterfly;monarch|Danaus plexippus|Insecta|Lepidoptera|Nymphalidae|0.00027-0.00075|0.04-0.05|0.1-0.7|40|Least Concern|the Americas|wingspan=8.9–10.2 cm (3.5–4 in);note=the migratory generation lives up to about 8 months;diet=nectar (caterpillars eat milkweed)
red imported fire ant;ant|Solenopsis invicta|Insecta|Hymenoptera|Formicidae|0.000001-0.000003|0.002-0.006|0.1-0.5|-|Least Concern|native to South America|diet=omnivore
dragonfly;common green darner|Anax junius|Insecta|Odonata|Aeshnidae|0.001-0.0015|0.068-0.08|0.1-0.5|54|Least Concern|North America|wingspan=about 10 cm;diet=flying insects
moon jellyfish;jellyfish|Aurelia aurita|Scyphozoa|Semaeostomeae|Ulmaridae|-|0.25-0.4|0.5-1|-|Not Evaluated|coastal waters worldwide|note=about 95–98% water; its body fluid has a protein content well under 1%;diet=plankton
common octopus;octopus|Octopus vulgaris|Cephalopoda|Octopoda|Octopodidae|3-10|0.3-1.0|1-2|40|Least Concern|temperate and tropical seas|note=three hearts and blue blood;diet=crabs and molluscs
giant squid|Architeuthis dux|Cephalopoda|Oegopsida|Architeuthidae|150-275|10-13|5|-|Least Concern|deep oceans worldwide|note=has the largest eyes of any animal, about 27 cm across;diet=fish and squid
";

fn species() -> &'static Vec<Species> {
    static S: OnceLock<Vec<Species>> = OnceLock::new();
    S.get_or_init(|| {
        SPECIES
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                if f.len() < 11 {
                    return None;
                }
                let extra = f.get(11).map_or(vec![], |e| key_values(e));
                Some(Species {
                    names: f[0].split(';').map(String::from).collect(),
                    sci: f[1].into(),
                    class: f[2].into(),
                    order: f[3].into(),
                    family: f[4].into(),
                    mass: range(f[5]),
                    length: range(f[6]),
                    life: range(f[7]),
                    speed: f[8].parse().ok(),
                    status: f[9].into(),
                    range: f[10].into(),
                    extra,
                })
            })
            .collect()
    })
}

/// Common names that also mean other things (a country, a car, food) are
/// only recognized inside an animal question, not on their own.
const NOT_BARE: &[&str] = &["puma", "crow", "swift", "monarch", "ant", "panda", "lynx", "camel", "emu", "rhino", "anaconda", "bison", "beaver", "badger", "hare", "squirrel", "mallard", "honey bee", "honeybee", "dragonfly", "iguana"];

fn find_species(s: &str) -> Option<&'static Species> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("a ").trim_start_matches("an ").trim();
    let s = s.trim_end_matches("'s").trim_end_matches('s');
    let s1 = format!("{s}s");
    species().iter().find(|sp| sp.names.iter().any(|n| n == s || *n == s1) || sp.sci.to_lowercase() == s)
}

fn species_rows(sp: &Species) -> Vec<(String, String)> {
    let mut r = vec![("scientific name".to_string(), sp.sci.clone()), ("taxonomy".into(), format!("class {} | order {} | family {}", sp.class, sp.order, sp.family))];
    if let Some(m) = sp.mass {
        r.push(("adult mass".into(), kg_text(m)));
    }
    if let Some(h) = sp.get("height") {
        r.push(("height".into(), h.into()));
    }
    if let Some(l) = sp.length {
        r.push(("length".into(), m_text(l)));
    }
    if let Some(w) = sp.get("wingspan") {
        r.push(("wingspan".into(), w.into()));
    }
    if let Some(v) = sp.speed {
        r.push(("top speed".into(), kmh(v)));
    }
    if let Some(d) = sp.get("dive") {
        r.push(("diving speed".into(), d.into()));
    }
    if let Some(j) = sp.get("jump") {
        r.push(("jump".into(), j.into()));
    }
    if let Some(l) = sp.life {
        r.push(("lifespan".into(), format!("{}{}", span(l, "years"), if sp.status == "domesticated" { "" } else { " (in the wild)" })));
    }
    if let Some(d) = sp.get("diet") {
        r.push(("diet".into(), d.into()));
    }
    r.push(("range".into(), sp.range.clone()));
    r.push(("conservation status".into(), sp.status.clone()));
    if let Some(note) = sp.get("note") {
        r.push(("note".into(), note.into()));
    }
    r
}

fn species_card(sp: &Species) -> Answer {
    let mut lead = vec![];
    if let Some(m) = sp.mass {
        lead.push(format!("mass {}", span(m, "kg")));
    }
    if let Some(v) = sp.speed {
        lead.push(format!("top speed {} km/h", n(v)));
    }
    if let Some(l) = sp.life {
        lead.push(format!("lifespan {}", span(l, "years")));
    }
    Answer::new("animal", vec![
        input(format!("{} ({})", sp.name(), sp.sci)),
        primary(pod("Result", "Basic properties", text(format!("{} — {}", sp.sci, lead.join(", "))))),
        pod("Properties", "Properties", rows(species_rows(sp))),
    ])
}

/// One property of a species as a short text, by question word.
fn species_prop(sp: &Species, what: &str) -> Option<(String, String)> {
    let (label, v) = match what {
        "speed" | "run" | "land speed" | "top speed" => ("top speed", sp.speed.map(kmh)?),
        "fly" | "air speed" | "airspeed" | "flight speed" | "dive" => ("flight speed", match sp.get("dive") {
            Some(d) => format!("{d}; about {} in level flight", kmh(sp.speed?)),
            None => kmh(sp.speed?),
        }),
        "swim" | "swimming speed" => ("swimming speed", sp.get("swim").map(String::from).or_else(|| sp.speed.map(kmh))?),
        "jump" => ("jump", sp.get("jump")?.to_string()),
        "wingspan" => ("wingspan", sp.get("wingspan")?.to_string()),
        "lifespan" | "life span" | "life expectancy" | "age" => ("lifespan", format!("{}{}", span(sp.life?, "years"), if sp.status == "domesticated" { "" } else { " (in the wild)" })),
        "mass" | "weight" => ("adult mass", kg_text(sp.mass?)),
        "length" | "size" => ("length", m_text(sp.length?)),
        "height" => ("height", sp.get("height").map(String::from).or_else(|| sp.length.map(m_text))?),
        "diet" | "food" => ("diet", sp.get("diet")?.to_string()),
        "range" | "habitat" => ("range", sp.range.clone()),
        "conservation status" | "status" => ("conservation status", sp.status.clone()),
        "scientific name" | "latin name" => ("scientific name", sp.sci.clone()),
        "taxonomy" | "classification" | "taxonomic classification" => ("taxonomy", format!("Animalia > Chordata > {} > {} > {} > {}", sp.class, sp.order, sp.family, sp.sci)),
        _ => return None,
    };
    Some((label.to_string(), v))
}

fn prop_word(w: &str) -> &str {
    match w {
        "how fast can" | "how fast does" | "how fast is" => "speed",
        "how far can" | "how high can" => "jump",
        "how long does" | "how long do" => "lifespan",
        "how big is" | "how big are" => "length",
        "how heavy is" | "how heavy are" | "how much does" => "mass",
        "average lifespan" | "lifespans" | "life expectancy" | "lifespan" => "lifespan",
        "top speed" | "maximum speed" | "speed" => "speed",
        _ => w,
    }
}

fn species_question(l: &str) -> Option<Answer> {
    // "how fast can a giraffe run", "how far can a tiger jump"
    if let Some(m) = re!(r"^(how fast can|how fast does|how fast is|how far can|how high can|how long does|how long do|how heavy is|how heavy are|how big is|how big are) (?:an? |the )?(.+?)(?: (run|fly|swim|jump|live|weigh|get|be))?$").captures(l) {
        let sp = find_species(&m[2]).or_else(|| animal_alias(&m[2]))?;
        let verb = m.get(3).map_or("", |v| v.as_str());
        let what = match verb {
            "fly" => "fly",
            "swim" => "swim",
            "jump" => "jump",
            "live" => "lifespan",
            "weigh" => "mass",
            _ => prop_word(&m[1]),
        };
        let (label, v) = species_prop(sp, what)?;
        return Some(Answer::new("animal", vec![input(format!("{} {label}", sp.name())), primary(pod("Result", &cap(&label), text(v))), pod("Properties", "Properties", rows(species_rows(sp)))]));
    }
    // "air speed of a peregrine falcon vs land speed of a cheetah", "wingspan of the greater flamingo"
    let parts: Vec<&str> = re!(r"\s+(?:vs\.?|versus|compared to)\s+").split(l).collect();
    let mut out = vec![];
    for p in &parts {
        let m = re!(r"^(?:the )?(average lifespan|life expectancy|lifespan|life span|top speed|maximum speed|land speed|air speed|airspeed|flight speed|swimming speed|speed|wingspan|weight|mass|length|height|diet|habitat|range|conservation status|scientific name|latin name|taxonomy|taxonomic classification|classification) of (.+)$").captures(p)
            .or_else(|| re!(r"^(.+?) (lifespan|life span|top speed|speed|wingspan|weight|mass|length|height|diet|habitat|conservation status|scientific name|taxonomy|classification)$").captures(p).map(|_| re!(r"^(.+?) (lifespan|life span|top speed|speed|wingspan|weight|mass|length|height|diet|habitat|conservation status|scientific name|taxonomy|classification)$").captures(p).unwrap()))?;
        let (what, who) = if re!(r" of ").is_match(p) && m.get(2).map_or(false, |g| !["lifespan", "speed"].contains(&g.as_str())) && re!(r"^(?:the )?(average lifespan|life expectancy|lifespan|life span|top speed|maximum speed|land speed|air speed|airspeed|flight speed|swimming speed|speed|wingspan|weight|mass|length|height|diet|habitat|range|conservation status|scientific name|latin name|taxonomy|taxonomic classification|classification) of ").is_match(p) {
            (m[1].to_string(), m[2].to_string())
        } else {
            (m[2].to_string(), m[1].to_string())
        };
        // "lifespan of a horse, goat, and sheep"
        let names: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\band\b|&)\s*").split(&who).filter(|s| !s.trim().is_empty()).collect();
        for nm in names {
            let sp = find_species(nm).or_else(|| animal_alias(nm))?;
            let (label, v) = species_prop(sp, prop_word(&what))?;
            out.push((sp.name().to_string(), label, v));
        }
    }
    if out.is_empty() {
        return None;
    }
    if out.len() == 1 {
        let (who, label, v) = out.remove(0);
        let sp = find_species(&who).or_else(|| animal_alias(&who))?;
        return Some(Answer::new("animal", vec![input(format!("{who} {label}")), primary(pod("Result", &cap(&label), text(v))), pod("Properties", "Properties", rows(species_rows(sp)))]));
    }
    let main = out.iter().map(|(w, _, v)| format!("{w}: {v}")).collect::<Vec<_>>().join(" | ");
    let t = out.iter().map(|(w, lab, v)| vec![text(w.clone()), text(lab.clone()), text(v.clone())]).collect();
    Some(Answer::new("animals", vec![input(out.iter().map(|o| format!("{} {}", o.0, o.1)).collect::<Vec<_>>().join(" vs ")), primary(pod("Result", "Result", text(main))), pod("Table", "Comparison", table(Some(vec!["animal", "property", "value"]), t))]))
}

/// Domestic animals by their common name, mapped to a typical member.
fn animal_alias(s: &str) -> Option<&'static Species> {
    static D: OnceLock<Vec<Species>> = OnceLock::new();
    let d = D.get_or_init(|| {
        "horse|Equus caballus|Mammalia|Perissodactyla|Equidae|380-1000|2.0-2.4|25-30|88|domesticated|worldwide|height=1.4–1.8 m at the withers;diet=herbivore (grass, hay)
donkey|Equus asinus|Mammalia|Perissodactyla|Equidae|80-480|1.5-2.0|25-30|50|domesticated|worldwide|diet=herbivore
goat|Capra hircus|Mammalia|Artiodactyla|Bovidae|20-140|1.0-1.8|15-18|-|domesticated|worldwide|diet=herbivore (browse)
sheep|Ovis aries|Mammalia|Artiodactyla|Bovidae|45-160|1.2-1.8|10-12|40|domesticated|worldwide|diet=herbivore (grass)
cow;cattle|Bos taurus|Mammalia|Artiodactyla|Bovidae|500-1000|1.8-2.5|18-22|40|domesticated|worldwide|diet=herbivore (grass)
pig|Sus domesticus|Mammalia|Artiodactyla|Suidae|50-350|0.9-1.8|15-20|17|domesticated|worldwide|diet=omnivore
dog|Canis familiaris|Mammalia|Carnivora|Canidae|1-90|0.3-1.2|10-13|45|domesticated|worldwide|note=greyhounds reach about 72 km/h;diet=omnivore
cat|Felis catus|Mammalia|Carnivora|Felidae|3.6-4.5|0.46|12-18|48|domesticated|worldwide|jump=up to about 5–6 times its own length;diet=carnivore
rabbit|Oryctolagus cuniculus domesticus|Mammalia|Lagomorpha|Leporidae|1-6|0.2-0.5|8-12|56|domesticated|worldwide|diet=herbivore
chicken|Gallus gallus domesticus|Aves|Galliformes|Phasianidae|1.5-4|0.4-0.5|5-10|14|domesticated|worldwide|diet=omnivore (grain, insects)
turkey|Meleagris gallopavo|Aves|Galliformes|Phasianidae|2.5-11|0.76-1.25|3-5|40|Least Concern|North America (wild); worldwide (domestic)|wingspan=1.25–1.44 m;diet=omnivore
duck|Anas platyrhynchos domesticus|Aves|Anseriformes|Anatidae|1-4|0.5-0.65|8-12|-|domesticated|worldwide|diet=omnivore
llama|Lama glama|Mammalia|Artiodactyla|Camelidae|130-200|1.8-2.2|15-25|56|domesticated|Andes; worldwide|height=1.7–1.8 m to the head;diet=herbivore"
            .lines()
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                Some(Species {
                    names: f[0].split(';').map(String::from).collect(),
                    sci: f[1].into(),
                    class: f[2].into(),
                    order: f[3].into(),
                    family: f[4].into(),
                    mass: range(f[5]),
                    length: range(f[6]),
                    life: range(f[7]),
                    speed: f[8].parse().ok(),
                    status: f[9].into(),
                    range: f[10].into(),
                    extra: f.get(11).map_or(vec![], |e| key_values(e)),
                })
            })
            .collect()
    });
    let s = s.trim().trim_start_matches("the ").trim_start_matches("a ").trim_start_matches("an ").trim();
    let s1 = s.trim_end_matches('s');
    d.iter().find(|sp| sp.names.iter().any(|n| n == s || n == s1))
}

fn species_lookup(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("the ").trim_end_matches(" animal");
    // "puma, lion, red fox"
    let names: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\bvs\.?\b|\bversus\b|\band\b|&)\s*").split(l).filter(|s| !s.trim().is_empty()).collect();
    if names.len() >= 2 {
        let sps: Option<Vec<&Species>> = names.iter().map(|s| find_species(s)).collect();
        let sps = sps?;
        let col = |f: &dyn Fn(&Species) -> String| sps.iter().map(|s| text(f(s))).collect::<Vec<_>>();
        let mut t = vec![];
        let fields: Vec<(&str, Box<dyn Fn(&Species) -> String>)> = vec![
            ("scientific name", Box::new(|s: &Species| s.sci.clone())),
            ("family", Box::new(|s: &Species| s.family.clone())),
            ("mass", Box::new(|s: &Species| s.mass.map(|m| span(m, "kg")).unwrap_or_default())),
            ("length", Box::new(|s: &Species| s.length.map(|m| span(m, "m")).unwrap_or_default())),
            ("top speed", Box::new(|s: &Species| s.speed.map(|v| format!("{} km/h", n(v))).unwrap_or_default())),
            ("lifespan", Box::new(|s: &Species| s.life.map(|m| span(m, "years")).unwrap_or_default())),
            ("conservation status", Box::new(|s: &Species| s.status.clone())),
        ];
        for (k, f) in &fields {
            let mut r = vec![text(*k)];
            r.extend(col(f.as_ref()));
            t.push(r);
        }
        let mut head = vec![""];
        head.extend(sps.iter().map(|s| s.name()));
        let main = sps.iter().map(|s| format!("{}: {}", s.name(), s.mass.map(|m| span(m, "kg")).unwrap_or_else(|| s.sci.clone()))).collect::<Vec<_>>().join(" | ");
        return Some(Answer::new("animals", vec![input(sps.iter().map(|s| s.name()).collect::<Vec<_>>().join(" | ")), primary(pod("Result", "Adult mass", text(main))), pod("Table", "Comparison", table(Some(head), t))]));
    }
    let sp = match find_species(l) {
        Some(sp) => sp,
        // the familiar domestic animals on their own ("dog", "horse"), but not "chicken" or "turkey"
        None if matches!(l, "dog" | "dogs" | "cat" | "cats" | "horse" | "horses" | "cow" | "cows" | "cattle" | "pig" | "pigs" | "sheep" | "goat" | "goats" | "donkey" | "donkeys" | "llama" | "llamas" | "rabbit" | "rabbits") => animal_alias(l)?,
        None => return None,
    };
    let exact = l.trim_end_matches('s');
    if NOT_BARE.contains(&exact) {
        return None;
    }
    Some(species_card(sp))
}

// ---------------------------------------------------------------------------
// higher taxa
// ---------------------------------------------------------------------------

// name | rank | lineage | members | described species | note
const TAXA: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("Lepidoptera", "order", "Animalia > Arthropoda > Insecta", "butterflies and moths", "about 180,000", "wings covered in tiny scales; larvae are caterpillars; adults usually feed through a coiled proboscis"),
    ("Coleoptera", "order", "Animalia > Arthropoda > Insecta", "beetles", "about 400,000", "the largest order of animals; front wings hardened into elytra"),
    ("Hymenoptera", "order", "Animalia > Arthropoda > Insecta", "sawflies, wasps, bees and ants", "about 150,000", "many are social; females of many species have stings"),
    ("Diptera", "order", "Animalia > Arthropoda > Insecta", "true flies, mosquitoes and gnats", "about 160,000", "a single pair of flying wings; hind wings reduced to halteres"),
    ("Hemiptera", "order", "Animalia > Arthropoda > Insecta", "true bugs, aphids, cicadas", "about 80,000", "piercing-sucking mouthparts"),
    ("Orthoptera", "order", "Animalia > Arthropoda > Insecta", "grasshoppers, crickets and katydids", "about 28,000", "jumping hind legs; many make sounds"),
    ("Odonata", "order", "Animalia > Arthropoda > Insecta", "dragonflies and damselflies", "about 6,300", "aquatic larvae; predatory adults"),
    ("Araneae", "order", "Animalia > Arthropoda > Arachnida", "spiders", "about 50,000", "spin silk; nearly all are venomous predators"),
    ("Primates", "order", "Animalia > Chordata > Mammalia", "lemurs, monkeys, apes and humans", "about 500", "large brains; forward-facing eyes; grasping hands"),
    ("Carnivora", "order", "Animalia > Chordata > Mammalia", "cats, dogs, bears, seals, weasels", "about 290", "carnassial teeth"),
    ("Rodentia", "order", "Animalia > Chordata > Mammalia", "rodents", "about 2,500", "the largest order of mammals; ever-growing incisors"),
    ("Chiroptera", "order", "Animalia > Chordata > Mammalia", "bats", "about 1,450", "the only mammals capable of true flight"),
    ("Cetacea", "infraorder", "Animalia > Chordata > Mammalia > Artiodactyla", "whales, dolphins and porpoises", "about 90", "fully aquatic mammals"),
    ("Squamata", "order", "Animalia > Chordata > Reptilia", "lizards and snakes", "about 11,000", "shed their skin; movable quadrate bones"),
    ("Testudines", "order", "Animalia > Chordata > Reptilia", "turtles and tortoises", "about 360", "a bony shell grown from the ribs"),
    ("Crocodilia", "order", "Animalia > Chordata > Reptilia", "crocodiles, alligators, caimans and gharials", "about 27", "the closest living relatives of birds"),
    ("Anura", "order", "Animalia > Chordata > Amphibia", "frogs and toads", "about 7,700", "tailless adults; tadpole larvae"),
    ("Mammalia", "class", "Animalia > Chordata", "mammals", "about 6,500", "hair and mammary glands"),
    ("Aves", "class", "Animalia > Chordata", "birds", "about 11,000", "feathers, beaks and hard-shelled eggs"),
    ("Reptilia", "class", "Animalia > Chordata", "reptiles", "about 12,000", "scaly skin; amniotic eggs"),
    ("Amphibia", "class", "Animalia > Chordata", "amphibians", "about 8,700", "most have an aquatic larval stage"),
    ("Insecta", "class", "Animalia > Arthropoda", "insects", "about 1 million", "six legs and three body parts"),
    ("Actinopterygii", "class", "Animalia > Chordata", "ray-finned fishes", "about 32,000", "fins supported by bony rays"),
    ("Chondrichthyes", "class", "Animalia > Chordata", "sharks, rays and chimaeras", "about 1,200", "skeletons of cartilage"),
    ("Felidae", "family", "Animalia > Chordata > Mammalia > Carnivora", "cats", "41", "retractable claws in most species"),
    ("Canidae", "family", "Animalia > Chordata > Mammalia > Carnivora", "dogs, wolves, foxes and jackals", "about 37", ""),
    ("Parastenocarididae", "family", "Animalia > Arthropoda > Crustacea > Copepoda > Harpacticoida", "tiny worm-like copepods of groundwater and interstitial habitats", "a few hundred", "bodies usually under 0.5 mm long"),
    ("Archiacanthocephala", "class", "Animalia > Acanthocephala", "thorny-headed worms that parasitize birds and mammals", "about 150", "adults anchor in the host's gut with a spiny proboscis; insects serve as intermediate hosts"),
    ("Transtillaspis", "genus", "Animalia > Arthropoda > Insecta > Lepidoptera > Tortricidae > Tortricinae > Euliini", "tortrix moths of the Neotropics, chiefly the Andes", "about 100", "described by Razowski (1987)"),
];

fn taxa(l: &str) -> Option<Answer> {
    let l = l.trim_end_matches(" taxonomy").trim_start_matches("taxonomy of ").trim_start_matches("compare ");
    let names: Vec<&str> = re!(r"\s*(?:,|\band\b|\bvs\.?\b|\bversus\b|&)\s*").split(l).filter(|s| !s.trim().is_empty()).collect();
    let ts: Option<Vec<_>> = names.iter().map(|n| TAXA.iter().find(|t| t.0.eq_ignore_ascii_case(n.trim()))).collect();
    let ts = ts?;
    if ts.is_empty() {
        return None;
    }
    if ts.len() == 1 {
        let t = ts[0];
        let mut r = vec![("rank".into(), t.1.into()), ("lineage".into(), format!("{} > {}", t.2, t.0)), ("members".into(), t.3.into()), ("described species".into(), t.4.into())];
        if !t.5.is_empty() {
            r.push(("notes".into(), t.5.into()));
        }
        return Some(Answer::new("taxon", vec![input(format!("{} ({})", t.0, t.1)), primary(pod("Result", "Members", text(format!("{} ({} described species)", t.3, t.4)))), pod("Properties", "Taxonomy", rows(r))]));
    }
    let tab = ts.iter().map(|t| vec![text(t.0), text(t.1), text(t.3), text(t.4), text(t.2)]).collect();
    let main = ts.iter().map(|t| format!("{}: {} — {}", t.0, t.1, t.3)).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("taxa", vec![input(ts.iter().map(|t| t.0).collect::<Vec<_>>().join(" vs ")), primary(pod("Result", "Result", text(main))), pod("Table", "Comparison", table(Some(vec!["taxon", "rank", "members", "species", "lineage"]), tab))]))
}

// ---------------------------------------------------------------------------
// breeds
// ---------------------------------------------------------------------------

struct Breed {
    kind: &'static str,
    names: Vec<String>,
    origin: String,
    uses: String,
    male: Range,
    female: Range,
    height: Range,
    fheight: Range,
    life: Range,
    extra: Vec<(String, String)>,
}

impl Breed {
    fn name(&self) -> &str {
        &self.names[0]
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.extra.iter().find(|e| e.0 == k).map(|e| e.1.as_str())
    }
    fn weight(&self) -> Range {
        match (self.male, self.female) {
            (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.max(b.1))),
            (a, b) => a.or(b),
        }
    }
}

// kind | names | origin | uses | male kg | female kg | male height cm | female height cm | lifespan y | extras
const BREEDS: &str = "\
dog|Labrador Retriever;labrador;lab|Canada and the United Kingdom|retrieving, guide and assistance work, companion|29-36|25-32|57-62|55-60|10-12|temperament=friendly, outgoing, even-tempered;group=sporting
dog|German Shepherd;german shepherd dog;alsatian|Germany|herding, police and military work, guarding|30-40|22-32|60-65|55-60|9-13|temperament=confident, courageous, intelligent;group=herding
dog|Golden Retriever|Scotland|retrieving, guide work, companion|30-34|25-32|56-61|51-56|10-12|temperament=friendly, intelligent, devoted;group=sporting
dog|French Bulldog;frenchie|France|companion|9-14|8-13|28-33|28-33|10-12|temperament=playful, adaptable, alert;group=non-sporting
dog|Bulldog;english bulldog|England|companion (originally bull-baiting)|23-25|18-23|31-40|31-40|8-10|temperament=docile, friendly, courageous;group=non-sporting
dog|Standard Poodle;poodle|Germany and France|retrieving water fowl, companion|20-32|18-27|45-60|45-60|12-15|temperament=intelligent, active, proud;group=non-sporting
dog|Toy Poodle|France|companion|2-3|2-3|24-28|24-28|12-15|temperament=intelligent, alert;group=toy
dog|Beagle|England|scent hunting (rabbits), detection|10-11|9-10|36-41|33-38|12-15|temperament=merry, friendly, curious;group=hound
dog|Rottweiler|Germany|herding, guarding, police work|50-60|35-48|61-69|56-63|9-10|temperament=loyal, confident, protective;group=working
dog|Dachshund|Germany|hunting badgers, companion|7-15|7-15|20-23|20-23|12-16|temperament=clever, stubborn, lively;group=hound
dog|Yorkshire Terrier;yorkie|England|ratting, companion|2-3.2|2-3.2|18-20|18-20|13-16|temperament=bold, feisty, affectionate;group=toy
dog|Boxer|Germany|guarding, companion|30-32|25-29|57-63|53-60|10-12|temperament=playful, energetic, loyal;group=working
dog|Siberian Husky;husky|Siberia (Russia)|sled pulling, companion|20-27|16-23|54-60|50-56|12-14|temperament=outgoing, mischievous, gentle;group=working
dog|Doberman Pinscher;doberman;dobermann|Germany|guarding, police and military work|40-45|32-35|68-72|63-68|10-13|temperament=loyal, fearless, alert;group=working
dog|Great Dane|Germany|boar hunting (historically), companion|54-90|45-59|76-86|71-81|7-10|temperament=friendly, patient, dependable;group=working
dog|Pug|China|companion|6-8|6-8|30-36|25-30|12-15|temperament=charming, mischievous, loving;group=toy
dog|Chihuahua|Mexico|companion|1.5-3|1.5-3|15-23|15-23|14-16|temperament=charming, graceful, sassy;group=toy
dog|Pomeranian|Pomerania (Germany and Poland)|companion|1.9-3.5|1.9-3.5|18-24|18-24|12-16|temperament=inquisitive, bold, lively;group=toy
dog|Shih Tzu|China (Tibet)|companion|4-7.2|4-7.2|20-28|20-28|10-18|temperament=affectionate, playful, outgoing;group=toy
dog|Border Collie|Scotland and England|herding sheep|14-20|12-19|48-56|46-53|12-15|temperament=intelligent, energetic, tenacious;group=herding
dog|Australian Shepherd;aussie|United States|herding livestock|23-29|18-23|51-58|46-53|12-15|temperament=smart, work-oriented, exuberant;group=herding
dog|Cavalier King Charles Spaniel;cavalier|United Kingdom|companion|5.9-8.2|5.9-8.2|30-33|30-33|9-14|temperament=affectionate, gentle, graceful;group=toy
dog|Irish Wolfhound|Ireland|hunting wolves and elk (historically), companion|54-68|40-55|81-90|71-79|6-8|temperament=calm, dignified, generous;group=hound;note=the tallest dog breed
dog|English Mastiff;mastiff|England|guarding|73-100|54-77|76-86|70-80|6-10|temperament=courageous, dignified, good-natured;group=working;note=the heaviest dog breed
dog|Saint Bernard|Switzerland|alpine rescue, companion|64-82|54-64|70-90|65-80|8-10|temperament=playful, charming, inquisitive;group=working
dog|Newfoundland|Canada|water rescue, draught work|60-70|45-55|69-74|63-69|9-10|temperament=sweet, patient, devoted;group=working
dog|Bernese Mountain Dog|Switzerland|draught and farm work|38-50|36-48|64-70|58-66|7-10|temperament=good-natured, calm, strong;group=working
dog|Scottish Deerhound;deerhound|Scotland|coursing deer|39.5-50|34-43|76-81|71-76|8-11|temperament=gentle, dignified, polite;group=hound
dog|Great Pyrenees;pyrenean mountain dog|France and Spain|guarding livestock|50-54|39-50|69-81|64-74|10-12|temperament=patient, calm, smart;group=working
dog|Greyhound|England|coursing, racing|29-40|27-34|71-76|68-71|10-13|temperament=gentle, independent, noble;group=hound;speed=about 72 km/h (45 mph), the fastest dog
dog|Leonberger|Germany|companion, draught work|54-77|41-59|72-80|65-75|7-9|temperament=friendly, gentle, playful;group=working
dog|Anatolian Shepherd;kangal|Turkey|guarding livestock|50-65|40-55|74-81|71-79|11-13|temperament=loyal, independent, reserved;group=working
dog|Shiba Inu;shiba|Japan|hunting, companion|9-11|7-9|35-43|33-41|13-16|temperament=alert, active, attentive;group=non-sporting
dog|Akita|Japan|guarding, hunting|45-59|32-45|64-71|58-66|10-13|temperament=courageous, dignified, profoundly loyal;group=working
dog|Pembroke Welsh Corgi;corgi|Wales|herding cattle|10-14|10-13|25-30|25-30|12-13|temperament=affectionate, smart, alert;group=herding
dog|Maltese|Malta|companion|3-4|3-4|20-25|20-23|12-15|temperament=gentle, playful, charming;group=toy
dog|Papillon|France|companion|3.6-4.5|3.6-4.5|20-28|20-28|14-16|temperament=happy, alert, friendly;group=toy
dog|Miniature Schnauzer|Germany|ratting, companion|5-8|5-8|30-36|30-36|12-15|temperament=friendly, smart, obedient;group=terrier
dog|Bichon Frise|France and Belgium|companion|5-10|5-10|23-30|23-30|14-15|temperament=playful, curious, peppy;group=non-sporting
dog|Jack Russell Terrier;jack russell|England|fox hunting|6-8|6-8|25-38|25-38|13-16|temperament=energetic, fearless, intelligent;group=terrier
dog|Boston Terrier|United States|companion|5-11|5-11|38-43|38-43|11-13|temperament=friendly, bright, amusing;group=non-sporting
dog|Dalmatian|Croatia (Dalmatia)|carriage dog, firehouse mascot|23-32|20-29|56-61|54-59|11-13|temperament=dignified, smart, outgoing;group=non-sporting
dog|Basset Hound|France|scent hunting|23-29|20-27|33-38|30-36|12-13|temperament=patient, low-key, charming;group=hound
dog|Bloodhound|Belgium and France|scent tracking|46-50|36-45|64-69|58-64|10-12|temperament=friendly, independent, inquisitive;group=hound
dog|Weimaraner|Germany|hunting|30-40|25-35|59-70|57-65|10-13|temperament=friendly, fearless, obedient;group=sporting
dog|Vizsla|Hungary|pointing and retrieving|20-29|18-25|56-64|53-61|12-14|temperament=affectionate, gentle, energetic;group=sporting
dog|Cocker Spaniel;american cocker spaniel|United States|flushing game birds|11-14|9-13|36-39|34-37|10-14|temperament=gentle, smart, happy;group=sporting
dog|Shetland Sheepdog;sheltie|Scotland (Shetland Islands)|herding|6-12|6-12|33-41|33-41|12-14|temperament=playful, energetic, bright;group=herding
dog|Collie;rough collie|Scotland|herding|27-34|23-29|61-66|56-61|12-14|temperament=devoted, graceful, proud;group=herding
dog|Samoyed|Siberia (Russia)|herding reindeer, sled pulling|20-30|16-23|53-60|48-53|12-14|temperament=adaptable, friendly, gentle;group=working
dog|Alaskan Malamute;malamute|Alaska (United States)|sled pulling|36-43|32-38|61-66|56-61|10-14|temperament=affectionate, loyal, playful;group=working
dog|Chow Chow|China|guarding, companion|25-32|20-27|46-56|43-51|8-12|temperament=dignified, aloof, loyal;group=non-sporting
dog|Shar Pei;chinese shar-pei|China|guarding, fighting (historically)|20-27|18-25|46-51|44-49|8-12|temperament=loyal, independent, calm;group=non-sporting
dog|Whippet|England|racing, coursing rabbits|11-18|9-14|48-56|46-53|12-15|temperament=affectionate, playful, calm;group=hound
dog|Afghan Hound|Afghanistan|sight hunting|23-27|20-25|68-74|63-69|12-18|temperament=independent, sweet, dignified;group=hound
dog|Bull Terrier|England|ratting, companion|23-32|20-27|53-56|53-56|12-13|temperament=playful, charming, mischievous;group=terrier
dog|Pit Bull Terrier;american pit bull terrier;pit bull|United States|catch dog, companion|16-27|14-23|46-53|43-51|12-14|temperament=confident, eager to please, strong;group=terrier
cat|Abyssinian|Ethiopia (Abyssinia); developed in Britain|companion|3.5-5|3-4|-|-|12-15|temperament=active, playful, curious;coat=short, ticked ruddy, red, blue or fawn
cat|Siamese|Thailand|companion|4-6|2.5-4.5|-|-|15-20|temperament=vocal, social, affectionate, intelligent;coat=short; pale body with darker points (seal, chocolate, blue, lilac)
cat|Persian|Iran|companion|4-5.5|3-4.5|-|-|12-17|temperament=calm, quiet, gentle;coat=long and dense
cat|Maine Coon|United States (Maine)|companion, mousing|5.9-8.2|3.6-5.4|-|-|12-15|temperament=gentle, friendly, intelligent;coat=long, shaggy;note=one of the largest domestic cat breeds
cat|Ragdoll|United States|companion|6.8-9|4.5-6.8|-|-|12-17|temperament=docile, placid, affectionate;coat=semi-long, colorpoint
cat|British Shorthair|United Kingdom|companion|5.5-8|3.5-5.5|-|-|12-20|temperament=easygoing, calm, loyal;coat=short, dense (classically blue)
cat|British Longhair|United Kingdom|companion|5-8|4-6|-|-|12-15|temperament=calm, easygoing, affectionate;coat=semi-long, dense
cat|Bengal|United States|companion|4.5-6.8|3.6-5.4|-|-|12-16|temperament=active, curious, confident;coat=short, spotted or marbled
cat|Sphynx|Canada|companion|3.5-5.5|3-4|-|-|8-14|temperament=energetic, affectionate, extroverted;coat=hairless
cat|Scottish Fold|Scotland|companion|4-6|2.7-4|-|-|11-14|temperament=sweet, calm, adaptable;coat=short or long; folded ears
cat|Russian Blue|Russia|companion|3.5-7|3-5|-|-|15-20|temperament=gentle, quiet, reserved;coat=short, silver-blue
cat|Norwegian Forest Cat|Norway|companion|5.5-9|3.5-6|-|-|14-16|temperament=friendly, independent, calm;coat=long, water-resistant
cat|Siberian|Russia|companion|4.5-9|3.5-6|-|-|11-15|temperament=affectionate, playful, intelligent;coat=long, triple coat
cat|Devon Rex|England|companion|3-4.5|2.5-3.5|-|-|9-15|temperament=playful, mischievous, social;coat=short, curly
cat|Cornish Rex|England|companion|3-4.5|2.5-3.5|-|-|11-15|temperament=active, affectionate, curious;coat=short, wavy
cat|Singapura|Singapore|companion|2.5-3.5|1.8-2.5|-|-|11-15|temperament=lively, curious, affectionate;coat=short, ticked sepia;note=the smallest cat breed
cat|American Shorthair|United States|companion, mousing|5-7|3.5-5.5|-|-|15-20|temperament=easygoing, gentle, adaptable;coat=short (classically silver tabby)
cat|Burmese|Myanmar and Thailand|companion|4-6|3-4.5|-|-|10-17|temperament=affectionate, playful, people-oriented;coat=short, satin
cat|Birman|Myanmar; developed in France|companion|4.5-7|3-5|-|-|12-16|temperament=gentle, quiet, affectionate;coat=semi-long, colorpoint with white gloves
cat|Exotic Shorthair|United States|companion|4.5-6.5|3-5|-|-|12-15|temperament=calm, sweet, playful;coat=short, plush
cat|Savannah|United States|companion|5.5-11|3.5-7|-|-|12-20|temperament=active, curious, adventurous;coat=short, spotted
cat|Turkish Angora|Turkey|companion|3.5-4.5|2.5-3.5|-|-|12-18|temperament=playful, intelligent, affectionate;coat=semi-long, silky (often white)
cat|Egyptian Mau|Egypt|companion|3.5-5|2.5-3.5|-|-|12-15|temperament=loyal, active, reserved;coat=short, naturally spotted;note=one of the fastest domestic cats
cat|Oriental Shorthair|Thailand; developed in Britain|companion|4-5.5|2.5-4|-|-|12-15|temperament=talkative, curious, social;coat=short, many colors
cattle|Holstein Friesian;holstein;friesian|Netherlands and northern Germany|dairy|900-1100|580-700|-|140-150|-|milk=about 10,000 kg per lactation (the highest-yielding dairy breed);color=black and white (sometimes red and white);butterfat=3.7%
cattle|Jersey|Jersey (Channel Islands)|dairy|540-820|400-500|-|115-125|-|milk=about 6,000–7,000 kg per lactation;color=fawn to dark brown;butterfat=about 5% (very rich milk)
cattle|Guernsey|Guernsey (Channel Islands)|dairy|700-800|450-500|-|120-130|-|milk=about 6,000 kg per lactation;color=fawn and white;butterfat=4.5%
cattle|Brown Swiss|Switzerland|dairy|900-1000|600-650|-|130-140|-|milk=about 9,000 kg per lactation;color=grey-brown
cattle|Ayrshire|Scotland|dairy|700-900|450-600|-|125-135|-|milk=about 7,500 kg per lactation;color=red and white
cattle|Angus;aberdeen angus;black angus|Scotland|beef|850-1000|550-650|-|-|-|color=solid black (Red Angus is red);note=naturally polled (hornless); well-marbled beef
cattle|Hereford|England|beef|1000-1200|550-700|-|-|-|color=red with a white face
cattle|Brahman|United States (from Indian zebu cattle)|beef|725-1000|450-635|-|-|-|color=light grey to red or black;note=heat- and insect-tolerant; large hump and loose skin
cattle|Charolais|France|beef|1100-1400|700-900|-|-|-|color=white to cream
cattle|Limousin|France|beef|1000-1100|600-700|-|-|-|color=golden red
cattle|Simmental|Switzerland|beef and dairy|1100-1300|700-800|-|-|-|color=red and white or gold and white
cattle|Highland;highland cattle;scottish highland|Scotland|beef|800|500|-|-|-|color=red, black, yellow or dun; long shaggy coat and long horns
cattle|Texas Longhorn;longhorn|United States|beef|700-1000|400-500|-|-|-|color=various;note=horns can span over 2 m
cattle|Podolica|southern Italy|beef and draught|700-900|450-550|-|-|-|color=grey (calves are wheat-colored);note=hardy breed of the Podolian group
cattle|Sayaguesa|Spain (Sayago, Zamora)|beef and draught (historically)|800-900|500-550|-|-|-|color=black;note=an endangered native Spanish breed
cattle|Wagyu|Japan|beef|700-900|450-550|-|-|-|color=black or red;note=intense intramuscular marbling
horse|Arabian;arab|Arabian Peninsula|riding, endurance riding|360-450|360-450|145-155|140-150|25-30|color=bay, grey, chestnut, black;note=distinctive dished face and high tail carriage
horse|Akhal-Teke;akhal teke|Turkmenistan|racing, endurance, dressage|400-450|400-450|147-163|147-163|20-25|color=often a metallic sheen (buckskin, palomino, bay)
horse|Thoroughbred|England|racing, show jumping, eventing|450-500|450-500|157-173|157-173|25-28|color=bay, chestnut, brown, grey
horse|Morgan|United States|driving, riding, dressage, western and ranch work, trail, harness|400-550|400-550|145-157|145-157|25-30|color=bay, black, chestnut;note=one of the earliest American breeds, from the stallion Figure (1789)
horse|American Quarter Horse;quarter horse|United States|ranch work, racing (quarter mile), rodeo|431-544|431-544|142-163|142-163|25-35|color=sorrel and many others;note=sprints at up to about 88 km/h
horse|Clydesdale|Scotland|draught, parade|820-910|820-910|162-183|162-183|20-25|color=bay with white feathered legs
horse|Shire|England|draught|850-1100|800-1000|173-193|163-178|25-30|color=black, bay, grey;note=among the tallest and heaviest horse breeds
horse|Percheron|France|draught|860-1000|860-1000|160-180|160-180|25-30|color=grey or black
horse|Friesian|Netherlands|driving, dressage|600-700|600-700|150-170|150-170|16-20|color=black
horse|Falabella|Argentina|companion, driving|40-80|40-80|64-86|64-86|40-45|note=one of the smallest horse breeds
horse|American Miniature Horse;miniature horse|United States|companion, therapy, driving|70-110|70-110|60-86|60-86|25-35|note=no taller than 86 cm (34 in) at the withers
horse|Shetland Pony;shetland|Scotland (Shetland Islands)|riding for children, driving|180-230|180-230|71-107|71-107|25-30|note=very strong for its size
horse|Mustang|United States (feral)|riding, ranch work|360-450|360-450|142-152|142-152|25-30|note=free-roaming descendants of Spanish horses
horse|Appaloosa|United States|riding, ranch work|430-500|430-500|142-163|142-163|30|color=leopard-spotted coat
horse|Andalusian|Spain|riding, dressage|400-500|400-500|152-162|150-158|25-30|color=mostly grey
chicken|Plymouth Rock;barred rock|United States|dual purpose (eggs and meat)|3.4|2.9|-|-|6-10|eggs=about 200 brown eggs a year;color=barred black and white (also white, buff and others)
chicken|Cochin|China|ornamental, broody hens|5|3.9|-|-|8-10|eggs=about 150 brown eggs a year;color=buff, black, white, partridge;note=very fluffy feathering with feathered feet
chicken|New Hampshire;new hampshire red|United States|dual purpose|3.9|2.9|-|-|5-8|eggs=about 200 brown eggs a year;color=chestnut red
chicken|Rhode Island Red|United States|dual purpose|3.9|2.9|-|-|5-8|eggs=about 250 brown eggs a year;color=dark red
chicken|Leghorn|Italy|eggs|2.7|2.0|-|-|4-6|eggs=about 280–300 white eggs a year;color=usually white
chicken|Orpington|England|dual purpose|4.5|3.6|-|-|8-10|eggs=about 200 light brown eggs a year;color=buff, black, white, blue
chicken|Silkie|China|ornamental, broody hens|1.8|1.4|-|-|7-9|eggs=about 100 cream eggs a year;note=fur-like plumage, black skin and five toes
chicken|Brahma|United States (from Chinese stock)|meat, ornamental|5.4|4.3|-|-|5-8|eggs=about 150 brown eggs a year;note=one of the largest chicken breeds
chicken|Australorp|Australia|dual purpose|3.9|2.9|-|-|6-10|eggs=about 250 brown eggs a year;color=black
chicken|Wyandotte|United States|dual purpose|3.9|2.7|-|-|6-12|eggs=about 200 brown eggs a year;color=silver-laced and others
chicken|Sussex|England|dual purpose|4.1|3.2|-|-|8|eggs=about 250 cream eggs a year
turkey|Broad Breasted White|United States|commercial meat|20-23|8-11|-|-|-|note=the standard commercial turkey; cannot breed naturally
turkey|Broad Breasted Bronze|United States|meat|20-23|8-11|-|-|-|
turkey|Standard Bronze;bronze|United States|meat (heritage)|16|10|-|-|-|
turkey|Bourbon Red|United States|meat (heritage)|10.4|6.4|-|-|-|color=dark red with white wings and tail
turkey|Narragansett|United States|meat (heritage)|10.4|6.4|-|-|-|color=black, grey, tan and white
turkey|Black Spanish;norfolk black|England and Spain|meat (heritage)|10.4|6.4|-|-|-|color=black
turkey|Slate;blue slate|United States|meat (heritage)|10.4|6.4|-|-|-|color=ashy blue
turkey|White Holland|Netherlands|meat (heritage)|14.5|9|-|-|-|color=white
turkey|Royal Palm|United States|ornamental, small meat|7.5|4.5|-|-|-|color=white with black edging
turkey|Beltsville Small White|United States|meat (small)|7.7|4.5|-|-|-|
turkey|Midget White|United States|meat (small)|6.4|3.6|-|-|-|
pig|Large White;yorkshire|England (Yorkshire)|meat (pork and bacon)|300-450|250-350|-|-|-|litter=about 11–13 piglets;color=white, erect ears
pig|Danish Landrace;landrace|Denmark|meat (bacon)|300-350|250-300|-|-|-|litter=about 12–14 piglets;color=white, long body and lop ears
pig|Duroc|United States|meat|350-400|250-300|-|-|-|litter=about 9–10 piglets;color=golden to dark red
pig|Hampshire|United States (from English stock)|meat (lean)|300-350|250-300|-|-|-|litter=about 9–10 piglets;color=black with a white belt over the shoulders
pig|Berkshire|England|meat (premium pork)|270-300|200-250|-|-|-|litter=about 8–10 piglets;color=black with white points
pig|Pietrain|Belgium|meat (very lean)|250-300|200-250|-|-|-|litter=about 9 piglets;color=white with black spots
pig|Tamworth|England|meat (bacon)|250-370|200-300|-|-|-|litter=about 8–10 piglets;color=ginger;note=a long-legged, tall pig
pig|Mangalica;mangalitsa|Hungary|meat (lard)|200-300|150-250|-|-|-|litter=about 6 piglets;note=curly wool-like coat
pig|Meishan|China|meat|150-200|120-150|-|-|-|litter=about 15–16 piglets (one of the most prolific breeds)
pig|Gloucestershire Old Spots;old spots|England|meat|270-320|200-250|-|-|-|litter=about 10 piglets;color=white with black spots
pig|Kunekune|New Zealand|grazing, pet|60-200|60-200|-|-|-|litter=about 6–8 piglets
sheep|Merino|Spain|fine wool|80-110|45-70|-|-|-|wool=very fine white wool, about 17–24 µm
sheep|Suffolk|England|meat|110-160|80-110|-|-|-|wool=medium white wool;color=white with a black face and legs
sheep|Finnsheep;finnish landrace|Finland|meat and wool|70-90|55-70|-|-|-|wool=white (also black, brown, grey);note=very prolific: litters of 3–4 lambs are common
sheep|Targhee|United States (U.S. Sheep Experiment Station, Dubois, Idaho, 1926)|wool and meat|90-135|57-90|-|-|-|wool=fine white wool
sheep|Dorper|South Africa|meat|100-120|70-90|-|-|-|wool=hair sheep (sheds)
sheep|Romney|England|wool and meat|100-125|70-90|-|-|-|wool=long white wool (colored flocks also kept)
sheep|Jacob|United Kingdom|wool, ornamental|55-80|40-55|-|-|-|wool=colored: piebald white with brown or black patches;note=often four horns
sheep|Lincoln;lincoln longwool|England|wool and meat|113-159|90-113|-|-|-|wool=colored and white longwool, very lustrous
sheep|Wensleydale|England|wool|130-140|100-110|-|-|-|wool=colored and white longwool, lustrous ringlets
sheep|Icelandic|Iceland|meat, wool and milk|90-100|60-70|-|-|-|wool=colored and white dual coat
sheep|Karakul|Central Asia|pelts, wool and meat|80-90|50-70|-|-|-|wool=colored (black, grey, brown)
sheep|Shetland|Scotland (Shetland Islands)|wool|40-57|34-45|-|-|-|wool=colored and white, very soft
sheep|Texel|Netherlands|meat|100-140|70-90|-|-|-|wool=white
sheep|Corriedale|New Zealand|wool and meat|79-125|59-82|-|-|-|wool=white, medium
sheep|Navajo-Churro;churro|United States (from Spanish stock)|wool and meat|55-79|38-55|-|-|-|wool=colored and white
goat|Saanen|Switzerland|dairy|70-90|60-70|80-90|74-80|-|milk=about 1,000 kg per lactation (among the highest);color=white or cream
goat|Alpine;french alpine|France|dairy|70-80|60|81-91|76|-|milk=about 950 kg per lactation;color=many colors
goat|Toggenburg|Switzerland|dairy|68|54|71-81|66|-|milk=about 900 kg per lactation;color=light fawn to chocolate with white markings
goat|Nubian;anglo-nubian|England|dairy and meat|79|61|81-91|76|-|milk=about 800 kg per lactation, high butterfat;color=many colors;note=long, drooping ears and a Roman nose
goat|LaMancha|United States|dairy|68|59|76|71|-|milk=about 900 kg per lactation;note=very short ears
goat|Oberhasli|Switzerland|dairy|68|54|76-86|71-81|-|milk=about 800 kg per lactation;color=chamois: bay with black markings (belly, legs, dorsal stripe)
goat|Boer|South Africa|meat|110-135|90-100|80-90|70-80|-|color=white body with a red-brown head
goat|Nigerian Dwarf|United States (from West African stock)|dairy, pet|34|34|48-58|43-57|-|milk=about 350 kg per lactation, very rich;color=many colors
goat|Pygmy;pygmy goat|West Africa (Cameroon)|meat, pet|27-39|24-34|41-58|41-53|-|color=agouti, black, caramel
goat|Angora|Turkey|mohair|80-100|35-50|-|-|-|color=white;note=fleece yields about 4–5 kg of mohair a year
goat|Nera Verzasca|Switzerland (Verzasca valley, Ticino)|meat and milk|60-80|45-60|80-90|70-80|-|color=black;note=hardy mountain breed
goat|Kiko|New Zealand|meat|110-135|45-70|-|-|-|color=usually white
";

fn breeds() -> &'static Vec<Breed> {
    static B: OnceLock<Vec<Breed>> = OnceLock::new();
    B.get_or_init(|| {
        BREEDS
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                if f.len() < 9 {
                    return None;
                }
                let kind: &'static str = match f[0] {
                    "dog" => "dog",
                    "cat" => "cat",
                    "cattle" => "cattle",
                    "horse" => "horse",
                    "chicken" => "chicken",
                    "turkey" => "turkey",
                    "pig" => "pig",
                    "sheep" => "sheep",
                    _ => "goat",
                };
                Some(Breed {
                    kind,
                    names: f[1].split(';').map(String::from).collect(),
                    origin: f[2].into(),
                    uses: f[3].into(),
                    male: range(f[4]),
                    female: range(f[5]),
                    height: range(f[6]),
                    fheight: range(f[7]),
                    life: range(f[8]),
                    extra: f.get(9).map_or(vec![], |e| key_values(e)),
                })
            })
            .collect()
    })
}

fn kind_word(w: &str) -> Option<&'static str> {
    Some(match w.trim().trim_end_matches('s') {
        "dog" | "canine" | "puppie" | "puppy" => "dog",
        "cat" | "feline" | "kitten" => "cat",
        "cattle" | "cow" | "bull" | "beef cattle" | "dairy cattle" | "dairy cow" => "cattle",
        "horse" | "equine" | "pony" | "ponie" => "horse",
        "chicken" | "hen" | "rooster" | "poultry" => "chicken",
        "turkey" => "turkey",
        "pig" | "swine" | "hog" => "pig",
        "sheep" | "ram" | "ewe" => "sheep",
        "goat" => "goat",
        _ => return None,
    })
}

fn find_breed(s: &str, kind: Option<&str>) -> Option<&'static Breed> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("a ").trim_start_matches("an ").trim();
    let s = s.trim_end_matches("'s").trim();
    let (s, k) = match re!(r"^(.+?)\s+(dogs?|cats?|cattle|cows?|bulls?|horses?|chickens?|hens?|turkeys?|pigs?|swine|hogs?|sheep|goats?|pony|ponies)$").captures(s) {
        Some(c) if breeds().iter().any(|b| b.names.iter().any(|n| n.eq_ignore_ascii_case(&c[1]))) => (c[1].to_string(), kind_word(&c[2]).or(kind)),
        _ => (s.to_string(), kind),
    };
    let s1 = s.trim_end_matches('s');
    breeds().iter().find(|b| k.map_or(true, |k| k == b.kind) && b.names.iter().any(|n| n.eq_ignore_ascii_case(&s) || n.eq_ignore_ascii_case(s1)))
}

fn kind_plural(k: &str) -> &'static str {
    match k {
        "dog" => "dog",
        "cat" => "cat",
        "cattle" => "cattle",
        "horse" => "horse",
        "chicken" => "chicken",
        "turkey" => "turkey",
        "pig" => "pig",
        "sheep" => "sheep",
        _ => "goat",
    }
}

fn breed_rows(b: &Breed) -> Vec<(String, String)> {
    let mut r = vec![("animal".to_string(), b.kind.to_string()), ("origin".into(), b.origin.clone()), ("uses".into(), b.uses.clone())];
    if let Some(m) = b.male {
        r.push((if matches!(b.kind, "chicken") { "rooster weight" } else if b.kind == "turkey" { "tom weight" } else { "male weight" }.into(), kg_text(m)));
    }
    if let Some(m) = b.female {
        r.push((if matches!(b.kind, "chicken" | "turkey") { "hen weight" } else { "female weight" }.into(), kg_text(m)));
    }
    if let Some(h) = b.height {
        r.push(((if b.kind == "dog" || b.kind == "horse" { "male height at the withers" } else { "male height" }).into(), cm_text(h)));
    }
    if let Some(h) = b.fheight {
        r.push(("female height".into(), cm_text(h)));
    }
    if let Some(l) = b.life {
        r.push(("life expectancy".into(), span(l, "years")));
    }
    for (k, v) in &b.extra {
        let k = match k.as_str() {
            "milk" => "milk yield",
            "litter" => "litter size",
            "eggs" => "egg production",
            "coat" => "coat",
            "wool" => "wool",
            other => other,
        };
        r.push((k.to_string(), v.clone()));
    }
    r
}

fn breed_card(b: &Breed) -> Answer {
    let mut lead = vec![format!("origin {}", b.origin)];
    if let Some(w) = b.weight() {
        lead.push(format!("weight {}", span(w, "kg")));
    }
    if let Some(l) = b.life {
        lead.push(format!("life expectancy {}", span(l, "years")));
    }
    Answer::new("breed", vec![input(format!("{} ({} breed)", b.name(), b.kind)), primary(pod("Result", "Basic properties", text(lead.join(", ")))), pod("Properties", "Properties", rows(breed_rows(b)))])
}

/// A breed attribute as a number (kg, cm, years, per-year counts) for filters and rankings.
fn breed_value(b: &Breed, attr: &str, sex: &str) -> Option<f64> {
    match attr {
        "weight" | "mass" | "size" => match sex {
            "male" => mid(b.male),
            "female" => mid(b.female),
            _ => mid(b.weight()),
        },
        "height" | "tall" => match sex {
            "female" => mid(b.fheight),
            "male" => mid(b.height),
            _ => mid(b.height).or(mid(b.fheight)),
        },
        "lifespan" | "life expectancy" => mid(b.life),
        "milk" | "milk yield" => num_in(b.get("milk")?),
        "litter" | "litter size" => num_in(b.get("litter")?),
        "eggs" | "egg production" => num_in(b.get("eggs")?),
        _ => None,
    }
}

fn num_in(s: &str) -> Option<f64> {
    let c = re!(r"(\d[\d,]*(?:\.\d+)?)(?:\s*[–-]\s*(\d[\d,]*(?:\.\d+)?))?").captures(s)?;
    let a: f64 = c[1].replace(',', "").parse().ok()?;
    let b: f64 = c.get(2).map_or(Some(a), |x| x.as_str().replace(',', "").parse().ok())?;
    Some((a + b) / 2.0)
}

fn value_text(attr: &str, v: f64) -> String {
    match attr {
        "weight" | "mass" | "size" => format!("{} kg ({} lb)", n(v), n(v * 2.204_62)),
        "height" | "tall" => format!("{} cm ({} in)", n(v), n(v / 2.54)),
        "lifespan" | "life expectancy" => format!("{} years", n(v)),
        "milk" | "milk yield" => format!("{} kg per lactation", n(v)),
        "litter" | "litter size" => format!("{} piglets", n(v)),
        _ => format!("{} a year", n(v)),
    }
}

fn breed_queries(l: &str) -> Option<Answer> {
    // "list of dog breeds", "cat breeds"
    if let Some(m) = re!(r"^(?:list of |all |types of |kinds of )?(dog|cat|cattle|cow|horse|chicken|turkey|pig|sheep|goat)s? breeds$").captures(l) {
        let k = kind_word(&m[1])?;
        let bs: Vec<&Breed> = breeds().iter().filter(|b| b.kind == k).collect();
        let t = bs.iter().map(|b| vec![text(b.name()), text(b.origin.clone()), text(b.uses.clone()), text(b.weight().map(|w| span(w, "kg")).unwrap_or_default())]).collect();
        return Some(Answer::new("breeds", vec![input(format!("{} breeds", kind_plural(k))), primary(pod("Result", "Breeds", text(bs.iter().map(|b| b.name()).collect::<Vec<_>>().join(" | ")))), pod("Table", "Breeds", table(Some(vec!["breed", "origin", "uses", "weight"]), t))]));
    }
    // "cat breeds with mean weight < 8 lbs", "turkey breeds with male weight more than 20 pounds",
    // "goats with female height less than 23 inches", "dog breeds with mean male height under 12 inches"
    if let Some(m) = re!(r"^(?:(colored wool) )?(dog|cat|cattle|cow|horse|chicken|turkey|pig|sheep|goat)s?(?: breeds)? (?:with|that have|having) (?:an? )?(?:mean |average |typical )?(male |female |mean male |mean female |average male |average female )?(weight|height|lifespan|life expectancy|milk yield|litter size) (<|>|<=|>=|less than|more than|greater than|under|over|below|above|at least|at most) (\d+(?:\.\d+)?) ?(lbs?|pounds?|kg|kilograms?|inches|inch|in|cm|centimeters?|years?|hands?)?$").captures(l) {
        let k = kind_word(&m[2])?;
        let sex = m.get(3).map_or("", |s| s.as_str().trim().rsplit(' ').next().unwrap_or(""));
        let attr = &m[4];
        let mut v: f64 = m[6].parse().ok()?;
        let unit = m.get(7).map_or("", |u| u.as_str());
        v = match unit {
            u if u.starts_with("lb") || u.starts_with("pound") => v * 0.453_592,
            u if u.starts_with("inch") || u == "in" => v * 2.54,
            u if u.starts_with("hand") => v * 10.16,
            _ => v,
        };
        let less = matches!(&m[5], "<" | "<=" | "less than" | "under" | "below" | "at most");
        let colored = m.get(1).is_some();
        let mut hits: Vec<(&Breed, f64)> = breeds().iter().filter(|b| b.kind == k && (!colored || b.get("wool").map_or(false, |w| w.contains("colored")))).filter_map(|b| breed_value(b, attr, sex).map(|x| (b, x))).filter(|(_, x)| if less { *x < v } else { *x > v }).collect();
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if !less {
            hits.reverse();
        }
        let desc = format!("{}{} breeds with {}{} {} {}", if colored { "colored wool " } else { "" }, kind_plural(k), if sex.is_empty() { "mean ".to_string() } else { format!("{sex} ") }, attr, if less { "<" } else { ">" }, value_text(attr, v));
        if hits.is_empty() {
            return Some(Answer::new("breeds", vec![input(desc), primary(pod("Result", "Result", text("no breeds in the table match")))]));
        }
        let t = hits.iter().map(|(b, x)| vec![text(b.name()), text(value_text(attr, *x)), text(b.origin.clone())]).collect();
        return Some(Answer::new("breeds", vec![input(desc), primary(pod("Result", "Result", text(hits.iter().map(|(b, _)| b.name()).collect::<Vec<_>>().join(" | ")))), pod("Table", "Matching breeds", table(Some(vec!["breed", attr, "origin"]), t))]));
    }
    // rankings: "tallest dog breeds", "what are the smallest equines", "most milk producing cow",
    // "goats with highest milk yield", "top 3 ... by weight"
    let ranked = match re!(r"^(dog|cat|cattle|cow|horse|chicken|turkey|pig|sheep|goat)s?(?: breeds)? with (?:the )?(highest milk yield|most milk|largest litters?|highest weight|lowest weight|most eggs)$").captures(l) {
        Some(c) => {
            let w = match &c[2] {
                "highest milk yield" | "most milk" => "most milk producing",
                "largest litter" | "largest litters" => "most prolific",
                "highest weight" => "heaviest",
                "lowest weight" => "lightest",
                _ => "best egg laying",
            };
            format!("{w} {}", &c[1])
        }
        None => l.to_string(),
    };
    if let Some(m) = re!(r"^(?:what are |which are |what is |which is )?(?:the )?(?:top (\d+) )?(tallest|shortest|smallest|largest|biggest|heaviest|lightest|longest[- ]lived|most milk producing|highest milk yielding|most prolific|most productive egg laying|best egg laying)(?: domestic)? (dog|cat|cattle|cow|horse|equine|pony|chicken|hen|turkey|pig|sheep|goat)s?(?: breeds?)?$").captures(&ranked) {
        let (top, sup, kind) = (m.get(1).and_then(|g| g.as_str().parse::<usize>().ok()), m[2].to_string(), m[3].to_string());
        let k = kind_word(&kind)?;
        let (attr, high) = match sup.as_str() {
            "tallest" => ("height", true),
            "shortest" => ("height", false),
            "smallest" | "lightest" => ("weight", false),
            "largest" | "biggest" | "heaviest" => ("weight", true),
            s if s.starts_with("longest") => ("lifespan", true),
            "most milk producing" | "highest milk yielding" => ("milk", true),
            "most prolific" => ("litter", true),
            _ => ("eggs", true),
        };
        let attr = if k == "horse" && sup == "smallest" { "height" } else { attr };
        let mut v: Vec<(&Breed, f64)> = breeds().iter().filter(|b| b.kind == k).filter_map(|b| breed_value(b, attr, "").map(|x| (b, x))).collect();
        if v.is_empty() {
            return None;
        }
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if high {
            v.reverse();
        }
        v.truncate(top.unwrap_or(5));
        let t = v.iter().map(|(b, x)| vec![text(b.name()), text(value_text(attr, *x)), text(b.origin.clone())]).collect();
        return Some(Answer::new("breeds", vec![input(format!("{sup} {} breeds by {attr}", kind_plural(k))), primary(pod("Result", "Result", text(v.iter().map(|(b, _)| b.name()).collect::<Vec<_>>().join(" | ")))), pod("Table", "Ranking", table(Some(vec!["breed", attr, "origin"]), t))]));
    }
    // "Is a British Longhair cat > a Pomeranian's weight?"
    if let Some(m) = re!(r"^is (?:an? |the )?(.+?) (>|<|heavier than|lighter than|taller than|bigger than) (?:an? |the )?(.+?)(?:'s)?(?: (weight|height))?$").captures(l) {
        let a = find_breed(&m[1], None)?;
        let b = find_breed(&m[3], None)?;
        let attr = m.get(4).map_or(if m[2].starts_with("taller") { "height" } else { "weight" }, |g| g.as_str());
        let (x, y) = (breed_value(a, attr, "")?, breed_value(b, attr, "")?);
        let gt = matches!(&m[2], ">" | "heavier than" | "taller than" | "bigger than");
        let yes = if gt { x > y } else { x < y };
        return Some(Answer::new("breeds", vec![input(format!("{} {attr} {} {} {attr}", a.name(), if gt { ">" } else { "<" }, b.name())), primary(pod("Result", "Result", text(if yes { "True" } else { "False" }))), pod("Table", "Values", table(Some(vec!["breed", attr]), vec![vec![text(a.name()), text(value_text(attr, x))], vec![text(b.name()), text(value_text(attr, y))]]))]));
    }
    // "lifespan of a pug", "origin of Targhee sheep", "uses of Morgan horses", "what color is X",
    // "temperament of Siamese cats", "average litter size for Hampshire pig", "Oberhasli goat fiber color"
    let (what, who) = if let Some(m) = re!(r"^(?:what (?:are|is) )?(?:the )?(?:average |typical |mean )?(lifespan|life span|life expectancy|origin|uses?|colou?rs?|temperament|personality|litter size|milk yield|egg production|weight|height|size|wool|coat) (?:of|for) (?:an? |the )?(.+)$").captures(l) {
        (m[1].to_string(), m[2].to_string())
    } else if let Some(m) = re!(r"^what colou?r (?:is|are) (?:an? |the )?(.+)$").captures(l) {
        ("color".to_string(), m[1].to_string())
    } else if let Some(m) = re!(r"^(.+?) (?:fiber |fibre |coat |wool )?(colou?r|origin|lifespan|temperament|uses|litter size|milk yield)$").captures(l) {
        (m[2].to_string(), m[1].to_string())
    } else {
        (String::new(), String::new())
    };
    if !what.is_empty() {
        let b = find_breed(&who, None)?;
        let v = match what.trim_end_matches('s') {
            "lifespan" | "life span" | "life expectancy" => b.life.map(|l| span(l, "years"))?,
            "origin" => b.origin.clone(),
            "use" => b.uses.clone(),
            "color" | "colour" => b.get("color").or(b.get("coat")).or(b.get("wool"))?.to_string(),
            "temperament" | "personality" => b.get("temperament")?.to_string(),
            "litter size" => b.get("litter")?.to_string(),
            "milk yield" => b.get("milk")?.to_string(),
            "egg production" => b.get("eggs")?.to_string(),
            "wool" => b.get("wool")?.to_string(),
            "coat" => b.get("coat")?.to_string(),
            "weight" | "size" => kg_text(b.weight()?),
            "height" => cm_text(b.height.or(b.fheight)?),
            _ => return None,
        };
        return Some(Answer::new("breed", vec![input(format!("{} {} ({} breed)", b.name(), what, b.kind)), primary(pod("Result", &cap(what.trim_end_matches('s')), text(v))), pod("Properties", "Properties", rows(breed_rows(b)))]));
    }
    // "Jersey, Podolica and Brahman cattle", "Akhal teke and Arabian horse", "Cochin chicken and New Hampshire chicken"
    let (body, kind) = match re!(r"^(.+?)\s+(dogs?|cats?|cattle|cows?|horses?|chickens?|turkeys?|pigs?|sheep|goats?)$").captures(l) {
        Some(c) => (c[1].to_string(), kind_word(&c[2])),
        None => (l.to_string(), None),
    };
    let names: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\band\b|\bvs\.?\b|\bversus\b|&)\s*").split(&body).filter(|s| !s.trim().is_empty()).collect();
    let bs: Option<Vec<&Breed>> = names.iter().map(|s| find_breed(s, kind)).collect();
    let bs = bs?;
    if bs.len() == 1 {
        // a bare breed name must be unambiguous: "Jersey", "Morgan" or "Boxer" alone mean other things
        const AMBIGUOUS: &[&str] = &["jersey", "guernsey", "morgan", "boxer", "persian", "siamese", "bengal", "siberian", "arabian", "arab", "alpine", "shire", "lincoln", "sussex", "hampshire", "new hampshire", "friesian", "holstein", "slate", "bronze", "angus", "hereford", "wyandotte", "leghorn", "orpington", "tamworth", "berkshire", "yorkshire", "landrace", "mustang", "andalusian", "maltese", "akita", "shetland", "icelandic", "nubian", "angora", "brahma", "brahman", "highland", "longhorn", "lab", "collie", "bulldog", "beagle", "chihuahua", "husky", "cavalier", "mastiff", "newfoundland", "greyhound", "boer", "pygmy", "merino", "jacob", "romney", "texel", "karakul", "simmental", "limousin", "charolais", "wagyu", "cochin", "burmese", "savannah", "plymouth rock", "saanen", "kiko", "samoyed", "kangal", "papillon", "bloodhound", "thoroughbred", "pit bull", "aussie", "ayrshire", "suffolk", "lincoln longwool", "churro", "old spots", "australorp"];
        // bare names that are also places, people, words or well-known things need a kind
        if kind.is_none() && names.len() == 1 && AMBIGUOUS.contains(&names[0].trim()) {
            return None;
        }
        return Some(breed_card(bs[0]));
    }
    let mut t = vec![];
    let field = |label: &str, f: &dyn Fn(&Breed) -> String| {
        let mut r = vec![text(label)];
        r.extend(bs.iter().map(|b| text(f(b))));
        r
    };
    t.push(field("origin", &|b| b.origin.clone()));
    t.push(field("uses", &|b| b.uses.clone()));
    t.push(field("male weight", &|b| b.male.map(|m| span(m, "kg")).unwrap_or_default()));
    t.push(field("female weight", &|b| b.female.map(|m| span(m, "kg")).unwrap_or_default()));
    t.push(field("height", &|b| b.height.or(b.fheight).map(|m| span(m, "cm")).unwrap_or_default()));
    t.push(field("life expectancy", &|b| b.life.map(|m| span(m, "years")).unwrap_or_default()));
    let mut head = vec![""];
    head.extend(bs.iter().map(|b| b.name()));
    let main = bs.iter().map(|b| format!("{}: {}", b.name(), b.origin)).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("breeds", vec![input(bs.iter().map(|b| b.name()).collect::<Vec<_>>().join(" | ")), primary(pod("Result", "Origin", text(main))), pod("Table", "Comparison", table(Some(head), t))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    breed_queries(l).or_else(|| taxa(l)).or_else(|| species_question(l)).or_else(|| species_lookup(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_parse() {
        assert!(species().len() > 90);
        assert!(breeds().len() > 140);
        assert!(species().iter().all(|s| !s.sci.is_empty() && !s.status.is_empty()));
        assert!(breeds().iter().all(|b| b.weight().is_some()));
    }
}
