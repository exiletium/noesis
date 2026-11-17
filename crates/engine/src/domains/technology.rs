//! Machines and inventions: crewed missions and space probes, rovers,
//! telescopes and stations, rockets, ships and shipwrecks, aircraft,
//! nuclear tests, inventions and inventors, early computers and
//! programming languages, with attribute questions, comparisons and lists.

use super::animals::key_values;
use super::geo::find_city;
use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(String, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

struct Item {
    names: Vec<String>,
    kind: String,
    kv: Vec<(String, String)>,
    note: String,
}

impl Item {
    fn name(&self) -> String {
        self.names[0].clone()
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.kv.iter().find(|x| x.0 == k).map(|x| x.1.as_str())
    }
    fn num(&self, k: &str) -> Option<f64> {
        re!(r"\d[\d,]*(?:\.\d+)?").find(self.get(k)?)?.as_str().replace(',', "").parse().ok()
    }
}

// names (first is the display name, as written) | kind | key=value;… | note
const DATA: &str = "\
Apollo 11|crewed lunar landing mission|launch=16 July 1969;landing=20 July 1969 (Sea of Tranquility);return=24 July 1969;crew=Neil Armstrong, Buzz Aldrin, Michael Collins;rocket=Saturn V;duration=8 days 3 h 18 min;operator=NASA|the first crewed Moon landing
Apollo 13|crewed lunar mission (aborted)|launch=11 April 1970;return=17 April 1970;crew=Jim Lovell, Jack Swigert, Fred Haise;rocket=Saturn V;duration=5 days 22 h 54 min;operator=NASA|an oxygen tank exploded on the way to the Moon; the crew returned safely
Apollo 15|crewed lunar landing mission|launch=26 July 1971;landing=30 July 1971 (Hadley–Apennine);return=7 August 1971;crew=David Scott, Alfred Worden, James Irwin;rocket=Saturn V;duration=12 days 7 h 12 min;operator=NASA|the first mission with the Lunar Roving Vehicle
Vostok 1|crewed orbital flight|launch=12 April 1961;crew=Yuri Gagarin;duration=108 minutes (one orbit);rocket=Vostok-K;operator=Soviet space program|the first human spaceflight
Gemini 8;gemini viii|crewed orbital flight|launch=16 March 1966;crew=Neil Armstrong, David Scott;duration=10 h 41 min;rocket=Titan II GLV;operator=NASA|the first docking of two spacecraft in orbit (with an Agena target vehicle)
Voskhod 1|crewed orbital flight|launch=12 October 1964;crew=Vladimir Komarov, Konstantin Feoktistov, Boris Yegorov;duration=1 day 0 h 17 min;operator=Soviet space program|the first spaceflight with more than one crew member
Soyuz 36|crewed flight to Salyut 6|launch=26 May 1980;crew=Valery Kubasov, Bertalan Farkas;operator=Soviet space program (Intercosmos)|Bertalan Farkas was the first Hungarian in space
STS-2;sts 2;sts2|Space Shuttle mission|launch=12 November 1981;crew=Joe Engle, Richard Truly;orbiter=Columbia;duration=2 days 6 h 13 min;operator=NASA|the first time a crewed spacecraft flew to orbit a second time
Space Shuttle Columbia;columbia space shuttle;columbia;ov-102;columbia orbiter|Space Shuttle orbiter|first flight=12 April 1981 (STS-1);last flight=STS-107, lost on re-entry on 1 February 2003;missions=28;operator=NASA|the first Space Shuttle to fly in space; seven astronauts died in the 2003 disaster
Voyager 1|space probe|launch=5 September 1977;mass=825.5 kg;speed=17.0 km/s relative to the Sun (about 61,000 km/h);status=in interstellar space since 25 August 2012, the most distant human-made object;flybys=Jupiter (1979), Saturn (1980);operator=NASA / JPL|carries a Golden Record
Voyager 2|space probe|launch=20 August 1977;mass=825.5 kg;speed=15.4 km/s relative to the Sun (about 55,000 km/h);status=in interstellar space since 5 November 2018;flybys=Jupiter (1979), Saturn (1981), Uranus (1986), Neptune (1989);operator=NASA / JPL|the only probe to have visited Uranus and Neptune
Pioneer 10|space probe|launch=3 March 1972;mass=258 kg;flybys=Jupiter (3 December 1973);status=last signal received 23 January 2003; heading towards Aldebaran;operator=NASA|the first probe to cross the asteroid belt and to fly past Jupiter
Pioneer 11|space probe|launch=6 April 1973;mass=259 kg;flybys=Jupiter (1974), Saturn (1 September 1979);status=last contact 30 September 1995;operator=NASA|the first probe to fly past Saturn
New Horizons|space probe|launch=19 January 2006;mass=478 kg;flybys=Jupiter (2007), Pluto (14 July 2015), Arrokoth (1 January 2019);operator=NASA|the first spacecraft to visit Pluto
Cassini–Huygens;cassini;cassini-huygens;cassini huygens|Saturn orbiter and Titan lander|launch=15 October 1997;arrival=1 July 2004 (Saturn orbit);end=15 September 2017 (plunged into Saturn);mass=5,712 kg at launch (2,523 kg dry);fuel=about 33 kg of plutonium-238 (as plutonium dioxide) in three radioisotope thermoelectric generators;operator=NASA, ESA and ASI|the Huygens probe landed on Titan on 14 January 2005
NEAR Shoemaker;near;near shoemaker probe|asteroid orbiter|launch=17 February 1996;target=433 Eros;arrival=14 February 2000 (orbit of Eros);landing=12 February 2001 (on Eros);mass=805 kg at launch;operator=NASA / Johns Hopkins APL|the first spacecraft to orbit and land on an asteroid
Spirit rover;spirit;mer-a;spirit mars rover|Mars rover|launch=10 June 2003;landing=4 January 2004 (Gusev crater);end=last contact 22 March 2010 (mission ended 25 May 2011);mass=185 kg;wheels=6;distance=7.73 km driven;operator=NASA / JPL|
Opportunity rover;opportunity;mer-b;opportunity mars rover|Mars rover|launch=7 July 2003;landing=25 January 2004 (Meridiani Planum);end=last contact 10 June 2018 (mission ended 13 February 2019);mass=185 kg;wheels=6;distance=45.16 km driven;operator=NASA / JPL|planned for 90 sols, it worked for more than 14 years
Curiosity rover;curiosity;mars science laboratory;curiosity mars rover|Mars rover|launch=26 November 2011;landing=6 August 2012 (Gale crater);mass=899 kg;wheels=6;length=2.9 m;power=multi-mission radioisotope thermoelectric generator (about 110 W);operator=NASA / JPL|still exploring Mount Sharp
Perseverance rover;perseverance;mars 2020|Mars rover|launch=30 July 2020;landing=18 February 2021 (Jezero crater);mass=1,025 kg;wheels=6;length=3 m;operator=NASA / JPL|carried the Ingenuity helicopter, the first powered aircraft on another planet
Sojourner;sojourner rover|Mars rover|launch=4 December 1996 (Mars Pathfinder);landing=4 July 1997 (Ares Vallis);mass=11.5 kg;wheels=6;operator=NASA / JPL|the first rover on Mars
Hubble Space Telescope;hubble;hubble telescope;hst|space telescope|launch=24 April 1990 (STS-31, Discovery);orbit=low Earth orbit at about 540 km, inclination 28.5°;period=about 95 minutes;speed=about 7.6 km/s;mass=11,110 kg;length=13.2 m;mirror=2.4 m primary mirror;operator=NASA and ESA|serviced by five Space Shuttle missions
James Webb Space Telescope;jwst;webb telescope;james webb telescope|space telescope|launch=25 December 2021 (Ariane 5);orbit=halo orbit around the Sun–Earth L2 point, 1.5 million km from Earth;mass=6,161 kg;mirror=6.5 m segmented primary mirror;operator=NASA, ESA and CSA|observes in the infrared
International Space Station;iss;space station|space station|launch=20 November 1998 (first module, Zarya);orbit=low Earth orbit at about 400–420 km, inclination 51.64°;period=about 92.9 minutes;speed=about 7.66 km/s (27,600 km/h);mass=about 420,000 kg;length=109 m;operator=NASA, Roscosmos, ESA, JAXA and CSA|continuously crewed since 2 November 2000
Saturn V;saturn 5|super heavy-lift rocket|first launch=9 November 1967;launches=13;height=110.6 m;diameter=10.1 m;mass=2,970 t (3,270 short tons);payload=140,000 kg to low Earth orbit, 48,600 kg to the Moon;operator=NASA|the rocket of the Apollo Moon landings
N1;n-1 rocket;n1 rocket|super heavy-lift rocket|first launch=21 February 1969;launches=4 (all failed);height=105 m;diameter=17 m at the base;mass=2,750 t (3,030 short tons);payload=95,000 kg to low Earth orbit;operator=Soviet space program|the Soviet Moon rocket
Starship;spacex starship|super heavy-lift rocket|first launch=20 April 2023;height=121–124 m;diameter=9 m;mass=about 5,000 t (5,500 short tons);payload=over 100,000 kg to low Earth orbit (planned, reusable);operator=SpaceX|the largest and most powerful rocket flown
Space Launch System;sls|super heavy-lift rocket|first launch=16 November 2022 (Artemis I);height=98 m;diameter=8.4 m;mass=2,610 t (2,880 short tons);payload=95,000 kg to low Earth orbit;operator=NASA|
Ariane 5|heavy-lift rocket|first launch=4 June 1996;last launch=5 July 2023;launches=117;height=46–52 m;diameter=5.4 m;mass=777 t (857 short tons);payload=21,000 kg to low Earth orbit, about 10,500 kg to geostationary transfer orbit;operator=Arianespace (ESA)|launched the James Webb Space Telescope
Falcon 9|medium-lift rocket|first launch=4 June 2010;height=70 m;diameter=3.7 m;mass=549 t (605 short tons);payload=22,800 kg to low Earth orbit (expendable);operator=SpaceX|the first orbital rocket with a reusable first stage
RMS Titanic;titanic|ocean liner|launched=31 May 1911;maiden voyage=10 April 1912;sank=15 April 1912 (struck an iceberg at 23:40 on 14 April);length=269.1 m;tonnage=46,328 GRT;deaths=about 1,500;owner=White Star Line;wreck=3,800 m deep in the North Atlantic, about 600 km south-southeast of Newfoundland;coords=41.7325,-49.9469|
RMS Lusitania;lusitania|ocean liner|launched=7 June 1906;sank=7 May 1915 (torpedoed by the German submarine U-20 off the Old Head of Kinsale, Ireland);length=239.9 m;tonnage=31,550 GRT;deaths=1,198;owner=Cunard Line;coords=51.4167,-8.5500|its sinking turned opinion in the United States against Germany
SS Edmund Fitzgerald;edmund fitzgerald|lake freighter|launched=7 June 1958;sank=10 November 1975 (in a storm on Lake Superior);length=222 m;deaths=29 (the whole crew);wreck=160 m deep in Canadian waters, about 27 km from Whitefish Bay;coords=46.9983,-85.1083|the largest ship on the Great Lakes when launched
Costa Concordia|cruise ship|launched=2 September 2005;sank=13 January 2012 (capsized after striking a rock off Isola del Giglio, Italy);length=290.2 m;tonnage=114,137 GT;deaths=32;owner=Costa Crociere;coords=42.3650,10.9217|raised in 2014 and scrapped in Genoa
RMS Queen Elizabeth 2;queen elizabeth 2;qe2;rms queen elizabeth ii;queen elizabeth ii|ocean liner|launched=20 September 1967;maiden voyage=2 May 1969;retired=27 November 2008;length=293.5 m;tonnage=70,327 GT;owner=Cunard Line|now a floating hotel in Dubai
SS Norway;ss france;norway ship|ocean liner and cruise ship|launched=11 May 1960;length=315.7 m;tonnage=76,049 GT;retired=2003 (scrapped 2008);owner=French Line, then Norwegian Cruise Line|the longest passenger ship from 1961 to 2004
Disney Magic|cruise ship|entered service=30 July 1998;length=294 m;tonnage=about 84,000 GT;owner=Disney Cruise Line|
USS Ronald Reagan;cvn-76|Nimitz-class aircraft carrier|commissioned=12 July 2003;length=333 m (1,092 ft);displacement=about 101,000 long tons;aircraft=up to 90;owner=US Navy|
USS Theodore Roosevelt;cvn-71|Nimitz-class aircraft carrier|commissioned=25 October 1986;length=333 m (1,092 ft);displacement=about 104,000 long tons;aircraft=up to 90;owner=US Navy|
Admiral Graf Spee;graf spee|Deutschland-class heavy cruiser (pocket battleship)|launched=30 June 1934;sank=17 December 1939 (scuttled off Montevideo after the Battle of the River Plate);length=186 m;displacement=14,890 t standard;owner=Kriegsmarine|
Boeing 747;747;jumbo jet|wide-body airliner|first flight=9 February 1969;entered service=22 January 1970 (Pan Am);length=70.6 m (747-400), 76.3 m (747-8);wingspan=64.4 m (747-400), 68.4 m (747-8);maximum takeoff weight=396,890 kg (747-400), 447,700 kg (747-8);speed=cruise Mach 0.85 (about 920 km/h);range=about 13,450 km (747-400);built=1,574 (1968–2022)|the original jumbo jet
Boeing 767;767|wide-body airliner|first flight=26 September 1981;entered service=8 September 1982 (United Airlines);length=48.5–61.4 m;wingspan=47.6–51.9 m;maximum takeoff weight=186,880 kg (767-300ER);speed=cruise Mach 0.80 (about 850 km/h);range=about 11,000 km (767-300ER);built=over 1,300|
Airbus A380;a380|wide-body double-deck airliner|first flight=27 April 2005;entered service=25 October 2007 (Singapore Airlines);length=72.7 m;wingspan=79.8 m;maximum takeoff weight=575,000 kg;speed=cruise Mach 0.85;range=about 15,000 km;built=254 (2003–2021)|the largest passenger airliner
Concorde|supersonic airliner|first flight=2 March 1969;entered service=21 January 1976;retired=24 October 2003;length=61.66 m;wingspan=25.6 m;speed=Mach 2.04 (about 2,180 km/h);built=20|
F-14 Tomcat;f-14;f14;grumman f-14 tomcat|carrier-based fighter|first flight=21 December 1970;entered service=22 September 1974;retired=22 September 2006 (US Navy);length=19.1 m;wingspan=19.55 m spread, 11.65 m swept;speed=Mach 2.34;crew=2;built=712|variable-sweep wings
Wright Flyer|airplane|first flight=17 December 1903 (Kitty Hawk, North Carolina);wingspan=12.3 m;inventor=Wilbur and Orville Wright|the first sustained, controlled, powered flight
Trinity;trinity test;trinity nuclear test;gadget|nuclear test|date=16 July 1945;location=Jornada del Muerto desert, New Mexico, United States;yield=about 25 kt (2021 reanalysis: 24.8 kt; earlier estimates about 21 kt);burst=tower, 30 m (100 ft);device=Gadget, a plutonium implosion device;country=United States|the first nuclear explosion
Hiroshima bombing;little boy;hiroshima;hiroshima nuclear explosion;hiroshima bomb;atomic bombing of hiroshima|nuclear bombing|date=6 August 1945;location=Hiroshima, Japan;yield=about 15 kt;burst=air burst at about 580 m (1,900 ft);device=Little Boy, a uranium gun-type bomb;country=United States;deaths=about 70,000–140,000 by the end of 1945|
Nagasaki bombing;fat man;nagasaki;nagasaki nuclear explosion;nagasaki bomb;atomic bombing of nagasaki|nuclear bombing|date=9 August 1945;location=Nagasaki, Japan;yield=about 21 kt;burst=air burst at about 500 m (1,650 ft);device=Fat Man, a plutonium implosion bomb;country=United States;deaths=about 39,000–80,000 by the end of 1945|
Ranger Baker;operation ranger baker|nuclear test|date=28 January 1951;location=Frenchman Flat, Nevada Test Site, United States;yield=8 kt;burst=air drop, about 330 m (1,080 ft);country=United States|part of Operation Ranger, the first tests at the Nevada Test Site
Sandstone Zebra;operation sandstone zebra|nuclear test|date=15 May 1948;location=Runit Island, Enewetak Atoll;yield=18 kt;burst=tower, 61 m (200 ft);country=United States|
Ivy Mike|nuclear test|date=1 November 1952;location=Elugelab, Enewetak Atoll;yield=10.4 Mt;burst=surface;country=United States|the first full-scale thermonuclear (hydrogen bomb) test
Castle Bravo;largest nuclear test in the us;largest nuclear bomb detonated by the us;largest us nuclear test|nuclear test|date=1 March 1954;location=Bikini Atoll, Marshall Islands;yield=15 Mt;burst=surface;country=United States|the largest US nuclear explosion, much bigger than its predicted 6 Mt
Tsar Bomba;an602;largest nuclear bomb;largest nuclear explosion|nuclear test|date=30 October 1961;location=Novaya Zemlya, Soviet Union;yield=about 50 Mt;burst=air burst at about 4,000 m;country=Soviet Union|the most powerful explosion ever made by humans
Divider;last nuclear test by the us;last nuclear test by us;last us nuclear test|nuclear test|date=23 September 1992;location=Nevada Test Site, United States;yield=under 20 kt;burst=underground;country=United States|the last US nuclear test
North Korea 2017 test;last nuclear test by north korea;north korea's last nuclear test;north korean nuclear test 2017|nuclear test|date=3 September 2017;location=Punggye-ri, North Korea;yield=estimated 140–250 kt;burst=underground;country=North Korea|North Korea's sixth test, claimed to be a hydrogen bomb
China's first nuclear test;596;project 596|nuclear test|date=16 October 1964;location=Lop Nur, Xinjiang, China;yield=22 kt;burst=tower;country=China|
incandescent light bulb;light bulb;lightbulb;the light bulb|invention|invented=1879 (practical carbon-filament lamp);inventor=Thomas Edison (also Joseph Swan, 1878–1880)|earlier arc and incandescent lamps go back to Humphry Davy (1802)
mobile phone;cell phone;cellphone;the cell phone;mobile telephone|invention|invented=3 April 1973 (first handheld call);inventor=Martin Cooper (Motorola)|the first handset, the DynaTAC, weighed about 1.1 kg
airbag;air bag;the airbag|invention|invented=1951–1953 (patents);inventor=Walter Linderer (Germany, 1951) and John W. Hetrick (US, 1953)|first offered in production cars in 1973 (General Motors)
Rubik's Cube;rubiks cube;the rubik's cube|invention|invented=1974;inventor=Ernő Rubik|patented in 1975 and sold worldwide from 1980
catalytic converter;the catalytic converter|invention|invented=1950s (patent 1956);inventor=Eugene Houdry|standard on US cars from 1975; three-way converters from 1981
telephone;the telephone|invention|invented=1876 (patent of 7 March 1876);inventor=Alexander Graham Bell|
iPod;ipod|portable media player|invented=23 October 2001 (release);inventor=Apple (team led by Tony Fadell)|
magnetic tape|invention|invented=1928;inventor=Fritz Pfleumer|the basis of tape recording
sextant|navigation instrument|invented=1730–1731;inventor=John Hadley and Thomas Godfrey (independently)|
hydrofoil|invention|invented=1906 (first working hydrofoil boats);inventor=Enrico Forlanini|
winged keel|yacht keel|invented=1983;inventor=Ben Lexcen|used by Australia II to win the 1983 America's Cup
clavivox|electronic musical instrument|invented=1950 (patent filed 1951, granted 1956);inventor=Raymond Scott|
electric razor;electric shaver|invention|invented=1928 (patent; sold from 1931);inventor=Jacob Schick|
polygraph;lie detector|invention|invented=1921;inventor=John Augustus Larson|
Jacquard loom|invention|invented=1804;inventor=Joseph Marie Jacquard|its punched cards inspired early computing
printing press;movable type printing press|invention|invented=about 1440;inventor=Johannes Gutenberg|
steam engine|invention|invented=1712 (Newcomen engine); 1769 (Watt's separate condenser);inventor=Thomas Newcomen; James Watt|
telegraph;electric telegraph|invention|invented=1837;inventor=Samuel Morse (and, in Britain, William Cooke and Charles Wheatstone)|
transistor|invention|invented=December 1947;inventor=John Bardeen, Walter Brattain and William Shockley (Bell Labs)|
World Wide Web;the web;www|invention|invented=1989 (proposal); 1991 (public);inventor=Tim Berners-Lee (CERN)|
phonograph|invention|invented=1877;inventor=Thomas Edison|
dynamite|invention|invented=1867;inventor=Alfred Nobel|
automobile;car;the car;motor car|invention|invented=1885–1886 (Benz Patent-Motorwagen, patent of 29 January 1886);inventor=Karl Benz|
radio|invention|invented=1895;inventor=Guglielmo Marconi (building on Hertz, Tesla, Popov and others)|
television|invention|invented=1926 (mechanical); 1927 (all-electronic);inventor=John Logie Baird; Philo Farnsworth|
lightning rod|invention|invented=1752;inventor=Benjamin Franklin|
bifocals;bifocal glasses|invention|invented=about 1784;inventor=Benjamin Franklin|
Franklin stove|invention|invented=1742;inventor=Benjamin Franklin|
glass armonica|musical instrument|invented=1761;inventor=Benjamin Franklin|
flexible urinary catheter|invention|invented=1752;inventor=Benjamin Franklin|
ornithopter (flying machine);leonardo's flying machine|design|invented=about 1485–1490;inventor=Leonardo da Vinci|
aerial screw (helicopter design)|design|invented=about 1489;inventor=Leonardo da Vinci|
parachute design|design|invented=about 1485;inventor=Leonardo da Vinci|
armoured fighting vehicle (tank design)|design|invented=1487;inventor=Leonardo da Vinci|
self-propelled cart|design|invented=about 1478;inventor=Leonardo da Vinci|
diving suit design|design|invented=about 1500;inventor=Leonardo da Vinci|
viola organista|musical instrument design|invented=about 1488–1489;inventor=Leonardo da Vinci|
mechanical knight (robot)|design|invented=about 1495;inventor=Leonardo da Vinci|
Apple II;apple ii;apple 2|home computer|released=10 June 1977;cpu=MOS 6502 at 1.023 MHz;ram=4–48 KB;price=US$1,298 (4 KB model);developer=Steve Wozniak (Apple Computer)|
Atari 400|home computer|released=November 1979;cpu=MOS 6502B at 1.79 MHz (NTSC) or 1.77 MHz (PAL);ram=8 KB, later 16 KB;price=US$549.95;developer=Atari, Inc.|
Commodore 64;c64|home computer|released=August 1982;cpu=MOS 6510 at 1.023 MHz (NTSC);ram=64 KB;price=US$595;developer=Commodore|the best-selling single computer model
BASIC;basic programming language;basic language|programming language|released=1 May 1964;developer=John G. Kemeny and Thomas E. Kurtz (Dartmouth College)|Beginner's All-purpose Symbolic Instruction Code
Java;java programming language;java language|programming language|released=23 May 1995 (announced);developer=James Gosling (Sun Microsystems)|write once, run anywhere
C;c programming language;c language|programming language|released=1972;developer=Dennis Ritchie (Bell Labs)|
Python;python programming language;python language|programming language|released=20 February 1991;developer=Guido van Rossum|
FORTRAN;fortran|programming language|released=1957;developer=John Backus (IBM)|the first widely used high-level language
LISP;lisp|programming language|released=1958;developer=John McCarthy (MIT)|
COBOL;cobol|programming language|released=1959;developer=CODASYL committee (influenced by Grace Hopper's FLOW-MATIC)|
Git;git version control|version control system|released=7 April 2005;developer=Linus Torvalds;language=C, shell, Perl|distributed version control
Mercurial;hg|version control system|released=19 April 2005;developer=Matt Mackall;language=Python, C, Rust|distributed version control";

fn items() -> &'static Vec<Item> {
    static I: OnceLock<Vec<Item>> = OnceLock::new();
    I.get_or_init(|| {
        DATA.lines()
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                if f.len() != 4 {
                    return None;
                }
                let names: Vec<String> = f[0].split(';').map(|s| s.trim().to_string()).collect();
                Some(Item { names, kind: f[1].to_string(), kv: key_values(f[2]), note: f[3].to_string() })
            })
            .collect()
    })
}

/// Bare words too ambiguous to answer on their own.
const NOT_BARE: &[&str] = &["c", "near", "columbia", "car", "the car", "radio", "television", "telephone", "java", "python", "git", "hg", "747", "767", "596", "trinity", "spirit", "opportunity", "curiosity", "perseverance", "hubble", "lisp", "the web", "www", "norway ship", "hiroshima", "nagasaki", "gadget", "n1", "mercurial", "divider", "fat man", "little boy", "starship", "jumbo jet"];

fn find(w: &str) -> Option<&'static Item> {
    let w = w.trim().trim_start_matches("the ").trim();
    let w = re!(r"\s+(?:nuclear explosion|nuclear test|nuke test|atomic test|test|space probe|probe|spacecraft|mission|rocket|ship|wreck|airliner|aircraft|airplane|plane)$").replace(w, "");
    let w = w.trim();
    let hit = |w: &str| items().iter().find(|i| i.names.iter().any(|n| n.eq_ignore_ascii_case(w)));
    hit(w).or_else(|| hit(&format!("the {w}")))
}

fn label(k: &str) -> &str {
    match k {
        "burst" => "height of burst",
        "coords" => "wreck coordinates",
        "cpu" => "processor",
        "ram" => "memory",
        "fuel" => "nuclear fuel",
        "launched" => "launched",
        "launch" => "launch date",
        "landing" => "landing",
        "distance" => "distance driven",
        _ => k,
    }
}

fn card(i: &Item) -> Vec<(String, String)> {
    let mut r = vec![("type".to_string(), i.kind.clone())];
    r.extend(i.kv.iter().filter(|(k, _)| k != "coords").map(|(k, v)| (label(k).to_string(), v.clone())));
    if !i.note.is_empty() {
        r.push(("note".into(), i.note.clone()));
    }
    r
}

/// Map a property phrase to data keys, in order of preference.
fn prop_keys(p: &str) -> Option<&'static [&'static str]> {
    Some(match p.trim() {
        "mass" | "weight" | "launch mass" => &["mass", "maximum takeoff weight", "displacement"],
        "length" | "how long" => &["length"],
        "height" | "how tall" => &["height"],
        "diameter" | "width" => &["diameter"],
        "wingspan" => &["wingspan"],
        "speed" | "velocity" | "top speed" | "max speed" | "maximum speed" | "how fast" => &["speed"],
        "crew" | "crew members" | "astronauts" | "astronaut" | "cosmonaut" | "pilot" | "pilots" => &["crew"],
        "launch" | "launch date" | "launch dates" | "launched" | "date launched" => &["launch", "launched", "first launch", "first flight"],
        "landing" | "landing date" | "landing site" => &["landing"],
        "wheels" | "number of wheels" => &["wheels"],
        "yield" | "explosive yield" | "power" | "energy" => &["yield", "power"],
        "height of burst" | "burst height" | "detonation height" => &["burst"],
        "date" | "test date" | "detonation date" => &["date", "launch", "invented", "released"],
        "location" | "site" | "test site" | "place" => &["location", "wreck", "landing"],
        "inventor" | "inventors" | "creator" | "designer" | "developer" => &["inventor", "developer"],
        "invention date" | "year invented" | "invented" => &["invented", "released"],
        "release date" | "released" | "introduced" | "introduction" => &["released", "invented", "entered service", "first flight"],
        "processor" | "cpu" | "processor speed" | "clock speed" => &["cpu"],
        "ram" | "memory" => &["ram"],
        "price" | "cost" | "launch price" => &["price"],
        "tonnage" | "gross tonnage" | "displacement" => &["tonnage", "displacement"],
        "deaths" | "casualties" | "fatalities" | "death toll" | "victims" => &["deaths"],
        "sinking" | "sinking date" | "sank" => &["sank"],
        "wreck" | "wreck location" | "wreck site" => &["wreck", "coords"],
        "orbit" | "orbital path" | "orbital altitude" | "altitude" => &["orbit"],
        "period" | "orbital period" => &["period"],
        "nuclear fuel mass" | "nuclear fuel" | "fuel" | "fuel mass" | "plutonium" => &["fuel"],
        "payload" | "payload capacity" => &["payload"],
        "first flight" | "maiden flight" => &["first flight", "first launch", "launch"],
        "operator" | "agency" | "owner" => &["operator", "owner"],
        "duration" | "mission duration" => &["duration"],
        "status" | "current status" => &["status"],
        "range" => &["range"],
        "flybys" | "planets visited" => &["flybys"],
        "programming language" | "written in" | "implementation language" => &["language"],
        "number built" | "production" | "built" => &["built"],
        _ => return None,
    })
}

const PROPS: &str = r"mass|weight|launch mass|length|height|diameter|wingspan|speed|velocity|top speed|max speed|maximum speed|crew|crew members|astronauts|launch dates?|launch|landing date|landing site|landing|wheels|number of wheels|yield|explosive yield|height of burst|burst height|test date|date|location|test site|site|inventors?|creator|designer|developer|invention date|year invented|release date|introduced|processor speed|processor|cpu|clock speed|ram|memory|launch price|price|gross tonnage|tonnage|displacement|deaths|casualties|fatalities|death toll|sinking date|wreck location|wreck site|orbital path|orbital altitude|orbit|altitude|orbital period|period|nuclear fuel mass|nuclear fuel|fuel mass|payload capacity|payload|first flight|maiden flight|operator|agency|owner|mission duration|duration|current status|status|range|flybys|programming language|number built|wreck";

fn value(i: &Item, keys: &[&str]) -> Option<(String, String)> {
    keys.iter().find_map(|k| i.get(k).map(|v| (label(k).to_string(), v.to_string())))
}

fn split(s: &str) -> Vec<String> {
    re!(r"\s*(?:,|/|;|\bvs\.?\s|\bversus\b|\band\b|&)\s*").split(s.trim()).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn resolve(s: &str) -> Option<Vec<&'static Item>> {
    if let Some(i) = find(s) {
        return Some(vec![i]);
    }
    let parts = split(s);
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|p| find(p)).collect()
}

fn single(i: &Item, title: &str, v: String) -> Answer {
    Answer::new(&i.name(), vec![input(format!("{} | {title}", i.name())), primary(pod("Result", &capitalize(title), text(v))), pod("Properties", "Properties", rows(card(i)))])
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

fn many(is: &[&Item], title: &str, vals: Vec<(String, String)>) -> Answer {
    let main = is.iter().zip(&vals).map(|(i, v)| format!("{}: {}", i.name(), v.1)).collect::<Vec<_>>().join(" | ");
    let t = is.iter().zip(&vals).map(|(i, v)| vec![text(i.name()), text(v.1.clone())]).collect();
    Answer::new("comparison", vec![input(format!("{} | {title}", is.iter().map(|i| i.name()).collect::<Vec<_>>().join(", "))), primary(pod("Result", &capitalize(title), text(main))), pod("Table", "Comparison", table(Some(vec!["", title]), t))])
}

fn attributes(l: &str) -> Option<Answer> {
    static PRE: OnceLock<Regex> = OnceLock::new();
    static SUF: OnceLock<Regex> = OnceLock::new();
    let pre = PRE.get_or_init(|| Regex::new(&format!(r"^(?:what (?:is|was|are|were) )?(?:the )?({PROPS})(?: of| for| on)? (?:the )?(.+)$")).unwrap());
    let suf = SUF.get_or_init(|| Regex::new(&format!(r"^(?:the )?(.+?)(?:'s)? ({PROPS})$")).unwrap());
    let mut cands: Vec<(String, String)> = vec![];
    if let Some(c) = re!(r"^(?:how many )?wheels (?:does|did|do) (?:the )?(.+?) have$").captures(l) {
        cands.push(("wheels".into(), c[1].to_string()));
    }
    if let Some(c) = re!(r"^how fast (?:was|is) the (?:processor|cpu)(?: on| of| in) (?:the )?(.+)$").captures(l) {
        cands.push(("processor".into(), c[1].to_string()));
    }
    if let Some(c) = pre.captures(l) {
        cands.push((c[1].to_string(), c[2].to_string()));
    }
    if let Some(c) = suf.captures(l) {
        cands.push((c[2].to_string(), c[1].to_string()));
    }
    if let Some(c) = re!(r"^(?:how (?:fast|heavy|long|tall) (?:is|was|does|did)|how much does) (?:the )?(.+?)(?: weigh| go| travel| fly)?$").captures(l) {
        let p = if l.contains("fast") { "speed" } else if l.contains("heavy") || l.contains("weigh") { "mass" } else if l.contains("tall") { "height" } else { "length" };
        cands.push((p.to_string(), c[1].to_string()));
    }
    let (is, vals) = cands.iter().find_map(|(p, who)| {
        let keys = prop_keys(p)?;
        let is = resolve(who)?;
        let vals: Vec<(String, String)> = is.iter().map(|i| value(i, keys)).collect::<Option<_>>()?;
        Some((is, vals))
    })?;
    Some(if is.len() == 1 { single(is[0], &vals[0].0.clone(), vals[0].1.clone()) } else { many(&is, &vals[0].0.clone(), vals) })
}

fn when_who(l: &str) -> Option<Answer> {
    if let Some(c) = re!(r"^when (?:was|were|did) (?:the )?(.+?) (launched|invented|introduced|released|created|developed|built|first flown|first fly|sink|sunk|sank|tested|detonated|land|landed|commissioned)$").captures(l) {
        let i = find(&c[1])?;
        let keys: &[&str] = match &c[2] {
            "launched" => &["launch", "launched", "first launch", "first flight"],
            "invented" | "created" | "developed" => &["invented", "released"],
            "introduced" | "released" => &["released", "invented", "entered service", "first flight"],
            "built" => &["launched", "invented", "released", "first flight"],
            "first flown" | "first fly" => &["first flight", "first launch"],
            "sink" | "sunk" | "sank" => &["sank"],
            "land" | "landed" => &["landing"],
            "commissioned" => &["commissioned", "entered service"],
            _ => &["date"],
        };
        let (t, v) = value(i, keys)?;
        return Some(single(i, &t, v));
    }
    if let Some(c) = re!(r"^who (?:invented|created|developed|designed|made|built) (?:the )?(.+)$").captures(l) {
        let i = find(&c[1])?;
        let (t, v) = value(i, &["inventor", "developer"])?;
        return Some(single(i, &t, v));
    }
    // "inventions by Benjamin Franklin", "Leonardo da Vinci inventions"
    if let Some(c) = re!(r"^(?:inventions|designs) (?:by|of) (.+)$").captures(l).or_else(|| re!(r"^(?:what did )?(.+?)(?:'s)? (?:inventions|designs|invent)$").captures(l)) {
        let who = c[1].trim().trim_start_matches("the ");
        if who.len() < 4 {
            return None;
        }
        let list: Vec<&Item> = items().iter().filter(|i| i.get("inventor").map_or(false, |v| v.to_lowercase().contains(who))).collect();
        if list.is_empty() {
            return None;
        }
        let t = list.iter().map(|i| vec![text(i.name()), text(i.get("invented").unwrap_or("").to_string())]).collect();
        return Some(Answer::new("inventions", vec![input(format!("inventions by {}", capitalize(who))), primary(pod("Result", "Inventions", text(list.iter().map(|i| i.name()).collect::<Vec<_>>().join(" | ")))), pod("Table", "Details", table(Some(vec!["invention", "date"]), t))]));
    }
    None
}

fn lists(l: &str) -> Option<Answer> {
    let yield_kt = |i: &Item| -> Option<f64> {
        let v = i.get("yield")?;
        let v = v.to_lowercase();
        let c = re!(r"([\d.,]+)\s*(kt|mt)").captures(&v)?;
        let x: f64 = c[1].replace(',', "").parse().ok()?;
        Some(if &c[2] == "mt" { x * 1000.0 } else { x })
    };
    if re!(r"^(?:the )?(?:largest|biggest|most powerful) (?:nuclear|atomic) (?:explosions|tests|bombs|weapons tests)$").is_match(l) {
        let mut v: Vec<(&Item, f64)> = items().iter().filter_map(|i| yield_kt(i).map(|y| (i, y))).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let extra = [("Soviet test 219", "24 December 1962", "24.2 Mt", "Soviet Union"), ("Soviet test 147", "5 August 1962", "21.1 Mt", "Soviet Union"), ("Castle Yankee", "5 May 1954", "13.5 Mt", "United States"), ("Castle Romeo", "27 March 1954", "11 Mt", "United States")];
        let mut t: Vec<(f64, Vec<Content>)> = v.iter().map(|(i, y)| (*y, vec![text(i.name()), text(i.get("date").unwrap_or("")), text(i.get("yield").unwrap_or("")), text(i.get("country").unwrap_or(""))])).collect();
        for (a, d, y, c) in extra {
            let kt: f64 = y.trim_end_matches(" Mt").parse::<f64>().unwrap_or(0.0) * 1000.0;
            t.push((kt, vec![text(a), text(d), text(y), text(c)]));
        }
        t.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        return Some(Answer::new("largest nuclear explosions", vec![input("largest nuclear explosions"), primary(pod("Result", "Result", text("Tsar Bomba, about 50 Mt (30 October 1961, Novaya Zemlya)"))), pod("Table", "Largest nuclear explosions", table(Some(vec!["test", "date", "yield", "country"]), t.into_iter().filter(|x| x.0 >= 5000.0).map(|x| x.1).collect()))]));
    }
    if re!(r"^(?:the )?nuclear (?:explosions|tests|weapons tests) (?:in|by|of) china$|^chinese nuclear tests$").is_match(l) {
        let r = vec![("number of tests".to_string(), "45 (1964–1996)".to_string()), ("first test".into(), "16 October 1964 (Project 596, 22 kt)".into()), ("first thermonuclear test".into(), "17 June 1967 (Test No. 6, 3.3 Mt)".into()), ("largest test".into(), "17 November 1976, about 4 Mt".into()), ("last test".into(), "29 July 1996".into()), ("test site".into(), "Lop Nur, Xinjiang".into())];
        return Some(Answer::new("nuclear tests in China", vec![input("nuclear explosions in China"), primary(pod("Result", "Result", text("45 nuclear tests between 1964 and 1996"))), pod("Details", "Details", rows(r))]));
    }
    if re!(r"^(?:the )?(?:earliest|first|oldest) programming languages$").is_match(l) {
        let t = [("Plankalkül", "1942–1945 (designed)", "Konrad Zuse"), ("Short Code", "1949", "John Mauchly"), ("Autocode", "1952", "Alick Glennie"), ("FLOW-MATIC", "1955", "Grace Hopper"), ("FORTRAN", "1957", "John Backus (IBM)"), ("LISP", "1958", "John McCarthy"), ("ALGOL 58", "1958", "committee"), ("COBOL", "1959", "CODASYL committee"), ("BASIC", "1964", "John Kemeny and Thomas Kurtz")];
        return Some(Answer::new("earliest programming languages", vec![input("earliest programming languages"), primary(pod("Result", "Result", text(t.iter().map(|x| x.0).collect::<Vec<_>>().join(" | ")))), pod("Table", "Details", table(Some(vec!["language", "year", "creator"]), t.iter().map(|x| vec![text(x.0), text(x.1), text(x.2)]).collect()))]));
    }
    if re!(r"^(?:the )?(?:mars missions|missions to mars|mars probes|mars rovers|mars landers)$").is_match(l) {
        let t = [("Mariner 4", "1964–1965", "NASA", "first successful flyby"), ("Mars 3", "1971", "Soviet Union", "first soft landing (contact lost after 14.5 s)"), ("Mariner 9", "1971", "NASA", "first Mars orbiter"), ("Viking 1 and 2", "1975–1976", "NASA", "landers and orbiters"), ("Mars Pathfinder / Sojourner", "1996–1997", "NASA", "first rover"), ("Mars Global Surveyor", "1996–1997", "NASA", "orbiter"), ("Mars Express", "2003", "ESA", "orbiter"), ("Spirit and Opportunity", "2003–2004", "NASA", "rovers"), ("Mars Reconnaissance Orbiter", "2005–2006", "NASA", "orbiter"), ("Phoenix", "2007–2008", "NASA", "polar lander"), ("Curiosity", "2011–2012", "NASA", "rover"), ("MAVEN", "2013–2014", "NASA", "orbiter"), ("Mangalyaan (Mars Orbiter Mission)", "2013–2014", "ISRO", "orbiter"), ("InSight", "2018", "NASA", "lander"), ("Hope", "2020–2021", "UAE", "orbiter"), ("Tianwen-1 / Zhurong", "2020–2021", "CNSA", "orbiter, lander and rover"), ("Perseverance / Ingenuity", "2020–2021", "NASA", "rover and helicopter")];
        return Some(Answer::new("Mars missions", vec![input("missions to Mars"), primary(pod("Table", "Notable Mars missions", table(Some(vec!["mission", "launch–arrival", "agency", "type"]), t.iter().map(|x| vec![text(x.0), text(x.1), text(x.2), text(x.3)]).collect())))]));
    }
    if re!(r"^(?:the )?jupiter (?:flybys|fly-bys|missions|probes)$").is_match(l) {
        let t = [("Pioneer 10", "3 December 1973", "flyby"), ("Pioneer 11", "2 December 1974", "flyby"), ("Voyager 1", "5 March 1979", "flyby"), ("Voyager 2", "9 July 1979", "flyby"), ("Galileo", "7 December 1995", "orbiter and atmospheric probe"), ("Ulysses", "8 February 1992 and 4 February 2004", "flybys"), ("Cassini", "30 December 2000", "flyby"), ("New Horizons", "28 February 2007", "flyby"), ("Juno", "4 July 2016", "orbiter")];
        return Some(Answer::new("Jupiter missions", vec![input("missions to Jupiter"), primary(pod("Table", "Jupiter flybys and orbiters", table(Some(vec!["spacecraft", "closest approach / arrival", "type"]), t.iter().map(|x| vec![text(x.0), text(x.1), text(x.2)]).collect())))]));
    }
    // "rockets with mass > 3000 short tons"
    if let Some(c) = re!(r"^rockets (?:with|having) (?:a )?(mass|height|diameter|payload) (>|<|greater than|more than|over|above|less than|under|below) ([\d.,]+)\s*(short tons|tons|tonnes|t|metric tons|m|meters|metres|kg)?$").captures(l) {
        let v: f64 = c[3].replace(',', "").parse().ok()?;
        let unit = c.get(4).map_or("", |x| x.as_str());
        let key = &c[1];
        let to_base = |x: f64| match unit {
            "short tons" | "tons" => x * 0.907_185,
            "kg" => x / 1000.0,
            _ => x,
        };
        let limit = to_base(v);
        let greater = matches!(&c[2], ">" | "greater than" | "more than" | "over" | "above");
        let mut hits: Vec<(&Item, f64)> = items().iter().filter(|i| i.kind.contains("rocket")).filter_map(|i| {
            let x = if key == "payload" { i.num("payload").map(|kg| kg / 1000.0) } else { i.num(key) }?;
            Some((i, x))
        }).filter(|(_, x)| if greater { *x > limit } else { *x < limit }).collect();
        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let t = hits.iter().map(|(i, _)| vec![text(i.name()), text(i.get(key).unwrap_or("").to_string())]).collect();
        return Some(Answer::new("rockets", vec![input(format!("rockets with {key} {} {} {unit}", &c[2], &c[3])), primary(pod("Result", "Result", text(if hits.is_empty() { "none".into() } else { hits.iter().map(|x| x.0.name()).collect::<Vec<_>>().join(" | ") }))), pod("Table", "Details", table(Some(vec!["rocket", key]), t))]));
    }
    None
}

fn haversine(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (la1, lo1, la2, lo2) = (a.0.to_radians(), a.1.to_radians(), b.0.to_radians(), b.1.to_radians());
    let h = ((la2 - la1) / 2.0).sin().powi(2) + la1.cos() * la2.cos() * ((lo2 - lo1) / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().asin()
}

fn wreck_distance(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:how far is|distance (?:from|between)) (?:the )?(.+?)(?: wreck)? (?:from|to|and) (.+)$").captures(l)?;
    let i = find(&c[1])?;
    let coords = i.get("coords")?;
    let (a, b) = coords.split_once(',')?;
    let p = (a.parse::<f64>().ok()?, b.parse::<f64>().ok()?);
    let city = find_city(c[2].trim())?;
    let d = haversine(p, (city.lat, city.lon));
    Some(Answer::new("distance", vec![input(format!("distance from the {} wreck to {}", i.name(), city.name)), primary(pod("Result", "Distance", text(format!("{} km ({} miles)", crate::print::fmt_float(d, 4), crate::print::fmt_float(d / 1.609_344, 4))))), pod("Properties", "Properties", rows(card(i)))]))
}

fn lookup(l: &str) -> Option<Answer> {
    let is = resolve(l)?;
    if is.len() == 1 {
        let i = is[0];
        if NOT_BARE.contains(&l) || i.kind == "design" {
            return None;
        }
        return Some(Answer::new(&i.name(), vec![input(format!("{} ({})", i.name(), i.kind)), primary(pod("Properties", "Properties", rows(card(i))))]));
    }
    // comparison: union of keys in first-seen order
    let mut keys: Vec<String> = vec![];
    for i in &is {
        for (k, _) in &i.kv {
            if k != "coords" && !keys.contains(k) {
                keys.push(k.clone());
            }
        }
    }
    keys.truncate(8);
    let mut head = vec!["".to_string(), "type".to_string()];
    head.extend(keys.iter().map(|k| label(k).to_string()));
    let t = is
        .iter()
        .map(|i| {
            let mut r = vec![text(i.name()), text(i.kind.clone())];
            r.extend(keys.iter().map(|k| text(i.get(k).unwrap_or("").to_string())));
            r
        })
        .collect();
    let head_ref: Vec<&str> = head.iter().map(|s| s.as_str()).collect();
    Some(Answer::new("comparison", vec![input(is.iter().map(|i| i.name()).collect::<Vec<_>>().join(", ")), primary(pod("Table", "Comparison", table(Some(head_ref), t)))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    lists(l).or_else(|| when_who(l)).or_else(|| wreck_distance(l)).or_else(|| attributes(l)).or_else(|| lookup(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        assert_eq!(items().len(), DATA.lines().count());
        for i in items() {
            assert!(!i.kv.iter().any(|(k, _)| k.is_empty()), "{}", i.names[0]);
        }
    }
}
