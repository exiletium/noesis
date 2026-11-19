//! Famous structures: towers and skyscrapers, dams, bridges, tunnels,
//! roller coasters, castles, theatres, canals and theme parks, with their
//! dimensions and opening years, comparisons and lists by country.

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

struct Structure {
    names: Vec<String>,
    kind: String,
    location: String,
    country: String,
    opened: String,
    nums: Vec<(String, f64)>,
    note: String,
}

impl Structure {
    fn name(&self) -> &str {
        &self.names[0]
    }
    fn num(&self, k: &str) -> Option<f64> {
        self.nums.iter().find(|x| x.0 == k).map(|x| x.1)
    }
}

// names | type | location | country | opened | numbers (height, length, span m; area ha; seats; speed km/h; drop m; power MW; floors; depth m) | note
const DATA: &str = "\
Eiffel Tower|tower|Paris|France|1889|height=330;floors=3|built by Gustave Eiffel's company for the 1889 World's Fair; about 7,300 t of wrought iron; about 6 million visitors a year
Burj Khalifa|skyscraper|Dubai|United Arab Emirates|2010|height=828;floors=163|the tallest building in the world
Empire State Building|skyscraper|New York City|United States|1931|height=443;floors=102|roof height 381 m; antenna to 443 m
Statue of Liberty|statue|Liberty Island, New York Harbor|United States|1886|height=93|statue 46 m from heel to torch; a gift from France, designed by Bartholdi with a frame by Eiffel
CN Tower|tower|Toronto|Canada|1976|height=553.3|
Tokyo Skytree|tower|Tokyo|Japan|2012|height=634|the tallest tower in the world
Leaning Tower of Pisa|bell tower|Pisa|Italy|1372|height=56.67|leans about 4° after stabilization work (1990–2001)
Elizabeth Tower;big ben|clock tower|London|United Kingdom|1859|height=96|Big Ben is properly the name of its great bell
Space Needle|tower|Seattle|United States|1962|height=184|
Petronas Towers;petronas twin towers|skyscraper|Kuala Lumpur|Malaysia|1998|height=451.9;floors=88|
Taipei 101|skyscraper|Taipei|Taiwan|2004|height=508;floors=101|
Shanghai Tower|skyscraper|Shanghai|China|2015|height=632;floors=128|
Willis Tower;sears tower|skyscraper|Chicago|United States|1973|height=442;floors=108|
Chrysler Building|skyscraper|New York City|United States|1930|height=318.9;floors=77|Art Deco; briefly the tallest building in the world (1930–1931)
Woolworth Building|skyscraper|New York City|United States|1913|height=241.4;floors=57|the tallest building in the world from 1913 to 1930
Flatiron Building|skyscraper|New York City|United States|1902|height=87;floors=22|
Central Park Tower|skyscraper|New York City|United States|2020|height=472.4;floors=98|the tallest residential building in the world
432 Park Avenue|skyscraper|New York City|United States|2015|height=425.7;floors=85|
John Hancock Center;875 north michigan avenue|skyscraper|Chicago|United States|1969|height=343.7;floors=100|
Trump International Hotel and Tower;trump tower chicago|skyscraper|Chicago|United States|2009|height=423.2;floors=98|
Transamerica Pyramid|skyscraper|San Francisco|United States|1972|height=260;floors=48|
Salesforce Tower|skyscraper|San Francisco|United States|2018|height=326;floors=61|
Lotte World Tower|skyscraper|Seoul|South Korea|2017|height=554.5;floors=123|
Ping An Finance Center;ping an finance centre|skyscraper|Shenzhen|China|2017|height=599.1;floors=115|
Abraj Al-Bait Clock Tower;makkah royal clock tower;abraj al bait|skyscraper|Mecca|Saudi Arabia|2012|height=601;floors=120|
Merdeka 118|skyscraper|Kuala Lumpur|Malaysia|2023|height=678.9;floors=118|the second-tallest building in the world
International Commerce Centre|skyscraper|Hong Kong|China|2010|height=484;floors=108|
The Shard;shard;shard london bridge|skyscraper|London|United Kingdom|2012|height=309.6|
30 St Mary Axe;the gherkin;gherkin|skyscraper|London|United Kingdom|2004|height=180;floors=41|
Washington Monument|obelisk|Washington, D.C.|United States|1884|height=169.3|the tallest stone structure and the tallest obelisk in the world
Gateway Arch|monument|St. Louis|United States|1965|height=192|stainless-steel catenary arch, 192 m tall and 192 m wide
Canton Tower|tower|Guangzhou|China|2010|height=600|
Ostankino Tower|tower|Moscow|Russia|1967|height=540.1|
Berlin TV Tower;fernsehturm berlin;berliner fernsehturm|tower|Berlin|Germany|1969|height=368|
One World Trade Center;freedom tower|skyscraper|New York City|United States|2014|height=541.3;floors=94|1,776 ft tall
Great Pyramid of Giza;great pyramid|pyramid|Giza|Egypt|c. 2560 BC|height=138.5|originally about 146.6 m; base 230.3 m on a side
Hoover Dam|dam|Black Canyon of the Colorado River, Nevada–Arizona|United States|1936|height=221;length=379;power=2080|impounds Lake Mead; concrete arch-gravity dam
Three Gorges Dam|dam|Yangtze River, Hubei|China|2006|height=181;length=2335;power=22500|the largest power station in the world by capacity; fully operational 2012
Grand Coulee Dam|dam|Columbia River, Washington|United States|1942|height=168;length=1592;power=6809|the largest power station in the United States
Glen Canyon Dam|dam|Colorado River, Arizona|United States|1966|height=216;length=475;power=1320|impounds Lake Powell
Mica Dam|dam|Columbia River, British Columbia|Canada|1973|height=243;length=792;power=2805|earth-fill dam impounding Kinbasket Lake
Itaipu Dam|dam|Paraná River|Brazil and Paraguay|1984|height=196;length=7919;power=14000|
Aswan High Dam|dam|Nile River, Aswan|Egypt|1970|height=111;length=3830;power=2100|impounds Lake Nasser
Jinping-I Dam|dam|Yalong River, Sichuan|China|2014|height=305;length=569;power=3600|the tallest dam in the world
Chivor Dam;la esmeralda dam|dam|Batá River, Boyacá|Colombia|1977|height=237;power=1000|rock-fill embankment dam forming the La Esmeralda reservoir
Golden Gate Bridge|suspension bridge|San Francisco Bay|United States|1937|length=2737;span=1280;height=227|tower height 227 m
Brooklyn Bridge|suspension bridge|East River, New York City|United States|1883|length=1834;span=486.3;height=84|
George Washington Bridge|suspension bridge|Hudson River, New York–New Jersey|United States|1931|length=1450;span=1067;height=184|the world's busiest motor-vehicle bridge
Bosphorus Bridge;15 july martyrs bridge|suspension bridge|Bosphorus, Istanbul|Turkey|1973|length=1560;span=1074;height=165|connects Europe and Asia
Forth Road Bridge|suspension bridge|Firth of Forth|United Kingdom|1964|length=2512;span=1006|
Forth Bridge|cantilever railway bridge|Firth of Forth|United Kingdom|1890|length=2467;span=521|a UNESCO World Heritage Site
Akashi Kaikyo Bridge;akashi-kaikyo bridge|suspension bridge|Akashi Strait|Japan|1998|length=3911;span=1991;height=282.8|
1915 Canakkale Bridge;canakkale bridge|suspension bridge|Dardanelles|Turkey|2022|length=4608;span=2023;height=318|the longest suspension bridge span in the world
Tower Bridge|bascule and suspension bridge|River Thames, London|United Kingdom|1894|length=244;span=61;height=65|
Sydney Harbour Bridge|steel arch bridge|Sydney Harbour|Australia|1932|length=1149;span=503;height=134|
Millau Viaduct|cable-stayed bridge|Tarn valley|France|2004|length=2460;span=342;height=343|the tallest bridge in the world
Danyang-Kunshan Grand Bridge|viaduct|Beijing–Shanghai High-Speed Railway, Jiangsu|China|2011|length=164800|the longest bridge in the world
Seikan Tunnel|railway tunnel|Tsugaru Strait|Japan|1988|length=53850;depth=240|23.3 km of it is under the seabed
Gotthard Base Tunnel|railway tunnel|Swiss Alps|Switzerland|2016|length=57104;depth=2300|the longest railway tunnel in the world
Channel Tunnel;chunnel|railway tunnel|Strait of Dover|United Kingdom and France|1994|length=50450;depth=75|37.9 km under the sea
Laerdal Tunnel;lærdal tunnel|road tunnel|Lærdal–Aurland|Norway|2000|length=24510|the longest road tunnel in the world
Moffat Tunnel|railway tunnel|Continental Divide, Colorado|United States|1928|length=9980|summit elevation 2,816 m
Holland Tunnel|road tunnel|Hudson River, New York–New Jersey|United States|1927|length=2608|
Lincoln Tunnel|road tunnel|Hudson River, New York–New Jersey|United States|1937|length=2504|
Eisenhower Tunnel;eisenhower-johnson tunnel|road tunnel|Interstate 70, Colorado|United States|1973|length=2730|at 3,401 m one of the highest vehicular tunnels
Kingda Ka|roller coaster|Six Flags Great Adventure, New Jersey|United States|2005|height=139;length=950;speed=206;drop=127|hydraulic launch; closed in 2024
Top Thrill Dragster;top thrill 2|roller coaster|Cedar Point, Ohio|United States|2003|height=128;length=853;speed=193;drop=122|
Formula Rossa|roller coaster|Ferrari World, Abu Dhabi|United Arab Emirates|2010|height=52;length=2200;speed=240|the fastest roller coaster in the world
Goliath|roller coaster|Six Flags Magic Mountain, California|United States|2000|height=72;length=1375;speed=137;drop=78|
Furius Baco|roller coaster|PortAventura Park, Salou|Spain|2007|height=14;length=850;speed=135|hydraulic launch to 135 km/h in 3 s
Millennium Force|roller coaster|Cedar Point, Ohio|United States|2000|height=94;length=2010;speed=150;drop=91|
Steel Dragon 2000|roller coaster|Nagashima Spa Land, Mie|Japan|2000|height=97;length=2479;speed=153;drop=93|the longest roller coaster in the world
Krak des Chevaliers|castle|Homs Governorate|Syria|1031|area=3|rebuilt by the Knights Hospitaller from 1142; a UNESCO World Heritage Site
Blarney Castle|castle|County Cork|Ireland|1446|height=27|home of the Blarney Stone
Egeskov Castle;egeskov|castle|Funen|Denmark|1554|area=20|Europe's best-preserved Renaissance water castle
Castillo de Cox;cox castle|castle|Cox, Alicante|Spain|15th century||a Moorish-origin castle rebuilt in the 15th–16th centuries above the town of Cox
Windsor Castle|castle|Windsor, Berkshire, England|United Kingdom|1070|area=5.3|the oldest and largest occupied castle in the world
Edinburgh Castle|castle|Edinburgh, Scotland|United Kingdom|12th century||stands on Castle Rock, a volcanic plug
Tower of London|castle|London, England|United Kingdom|1078|area=4.9|the White Tower was begun by William the Conqueror
Warwick Castle|castle|Warwick, England|United Kingdom|1068||
Caernarfon Castle|castle|Caernarfon, Wales|United Kingdom|1283||built by Edward I
Conwy Castle|castle|Conwy, Wales|United Kingdom|1289||built by Edward I
Stirling Castle|castle|Stirling, Scotland|United Kingdom|12th century||
Dover Castle|castle|Dover, England|United Kingdom|1180s||the largest castle in England
Leeds Castle|castle|Kent, England|United Kingdom|1119||
Alnwick Castle|castle|Northumberland, England|United Kingdom|1096||
Eilean Donan|castle|Kyle of Lochalsh, Scotland|United Kingdom|13th century||rebuilt 1912–1932
Neuschwanstein Castle;neuschwanstein|castle|Hohenschwangau, Bavaria|Germany|1886||built for King Ludwig II
Himeji Castle|castle|Himeji, Hyōgo|Japan|1609||a UNESCO World Heritage Site
Prague Castle|castle|Prague|Czech Republic|9th century|area=7|the largest ancient castle complex in the world
Bran Castle|castle|Bran, Brașov|Romania|1388||marketed as Dracula's Castle
Tovstonogov Bolshoi Drama Theater;bolshoi drama theater|theatre|Saint Petersburg|Russia|1919||named after its director Georgy Tovstonogov (1956–1989); housed in the former Maly Theatre building on the Fontanka (1878)
Solís Theatre;teatro solís;teatro solis|theatre|Montevideo|Uruguay|1856|seats=1500|Uruguay's principal theatre
Barbican Centre|arts centre|London|United Kingdom|1982|seats=1943|concert hall 1,943 seats; theatre 1,156 seats
Boston Opera House|theatre|Boston, Massachusetts|United States|1928|seats=2677|opened as the B. F. Keith Memorial Theatre
Muny;the muny;muny theater|outdoor theatre|Forest Park, St. Louis, Missouri|United States|1917|seats=11000|the largest outdoor musical theatre in the United States
Broadway Theatre;broadway theater|theatre|1681 Broadway, New York City|United States|1924|seats=1761|
Sydney Opera House|performing arts centre|Sydney|Australia|1973|seats=5738|Concert Hall seats 2,679
Royal Albert Hall|concert hall|London|United Kingdom|1871|seats=5272|
Metropolitan Opera House;the met|opera house|Lincoln Center, New York City|United States|1966|seats=3800|
Panama Canal|canal|Isthmus of Panama|Panama|1914|length=82000|
Suez Canal|canal|Isthmus of Suez|Egypt|1869|length=193300|
Erie Canal|canal|New York State|United States|1825|length=584000|
Kiel Canal|canal|Schleswig-Holstein|Germany|1895|length=98000|the world's busiest artificial waterway
Grand Canal;beijing-hangzhou grand canal|canal|Beijing to Hangzhou|China|5th century BC – 7th century AD|length=1794000|the longest canal in the world
Oxford Canal|canal|Oxford to Coventry (Hawkesbury Junction), England|United Kingdom|1790|length=126000|
Disneyland;disneyland park|theme park|Anaheim, California|United States|1955|area=34|
Tokyo Disneyland|theme park|Urayasu, Chiba|Japan|1983|area=51|
Magic Kingdom|theme park|Walt Disney World, Florida|United States|1971|area=43|
Disneyland Paris;disneyland park paris|theme park|Marne-la-Vallée|France|1992|area=57|
Mponeng Gold Mine;mponeng|mine|Gauteng|South Africa|1986|depth=4000|the deepest mine in the world
TauTona Mine;tautona|mine|Gauteng|South Africa|1962|depth=3900|
Kidd Mine|mine|Timmins, Ontario|Canada|1966|depth=2900|the deepest base-metal mine in the world
Three Mile Island;three mile island nuclear generating station;tmi|nuclear power plant|Londonderry Township, Pennsylvania|United States|1974|power=819|two pressurized-water reactors; Unit 2 partly melted down on March 28, 1979, the worst US commercial nuclear accident; Unit 1 shut down in 2019 (power is Unit 1's net output)
Diablo Canyon Power Plant;diablo canyon;diablo canyon nuclear power plant|nuclear power plant|Avila Beach, California|United States|1985|power=2256|two Westinghouse pressurized-water reactors of about 1,100 MW each (Unit 2 entered service in 1986); California's last nuclear plant
Angra 1;angra-1;angra-1 reactor;angra 1 reactor|nuclear power plant|Angra dos Reis, Rio de Janeiro|Brazil|1985|power=640|a Westinghouse pressurized-water reactor, Brazil's first nuclear power unit
La Crosse Boiling Water Reactor;lacrosse reactor;la crosse reactor;lacbwr|nuclear power plant|Genoa, Wisconsin|United States|1969|power=50|a small boiling-water reactor that ran from 1969 to 1987
Large Hadron Collider;lhc|particle accelerator|CERN, near Geneva|Switzerland and France|2008|length=26659|the world's largest and most powerful collider: protons at 6.8 TeV per beam, 100 m underground; found the Higgs boson in 2012
Relativistic Heavy Ion Collider;rhic|particle accelerator|Brookhaven National Laboratory, Upton, New York|United States|2000|length=3834|collides gold ions at up to 100 GeV per nucleon and polarized protons at up to 255 GeV; made the quark–gluon plasma
Stanford Linear Collider;slc|particle accelerator|SLAC, Menlo Park, California|United States|1989|length=3200|electron–positron collisions at about 91 GeV (the Z boson) from 1989 to 1998, using the 3.2 km SLAC linear accelerator
Tevatron|particle accelerator|Fermilab, Batavia, Illinois|United States|1983|length=6280|proton–antiproton collider at 0.98 TeV per beam; found the top quark in 1995; closed in 2011
Mount Rushmore;mount rushmore national memorial|memorial|Keystone, South Dakota|United States|1941|height=18|the heads of Washington, Jefferson, Theodore Roosevelt and Lincoln, each about 18 m (60 ft) tall, carved 1927–1941 under Gutzon and Lincoln Borglum
The Motherland Calls;motherland calls;rodina-mat zovyot|statue|Mamayev Kurgan, Volgograd|Russia|1967|height=85|commemorates the Battle of Stalingrad; 85 m to the tip of the sword
Great Buddha of Thailand;the great buddha of thailand;big buddha of thailand|statue|Wat Muang, Ang Thong Province|Thailand|2008|height=92|a seated Buddha painted gold, the tallest statue in Thailand
Big Lobster;the big lobster;larry the lobster|statue|Kingston SE, South Australia|Australia|1979|height=17|one of Australia's Big Things
Craigievar Castle|castle|Aberdeenshire, Scotland|United Kingdom|1626|height=18|a pink-harled tower house of seven storeys, finished for William Forbes in 1626
British Museum;the british museum|museum|Bloomsbury, London|United Kingdom|1759||founded 1753 and opened in 1759; a collection of about 8 million objects, among them the Rosetta Stone and the Parthenon sculptures
Arlington National Cemetery|cemetery|Arlington, Virginia|United States|1864|area=259|about 400,000 graves; the Tomb of the Unknown Soldier
Chalmette National Cemetery|cemetery|Chalmette, Louisiana|United States|1864|area=7|on the site of the 1815 Battle of New Orleans
Gettysburg Battlefield;gettysburg national military park|battlefield|Gettysburg, Pennsylvania|United States|1863|area=2441|site of the Battle of Gettysburg (July 1–3, 1863); national military park since 1895
Kennedy Compound|historic site|Hyannis Port, Massachusetts|United States|1926|area=2.4|three houses of the Kennedy family on Nantucket Sound; a National Historic Landmark since 1972
Yellowstone National Park;yellowstone|national park|Wyoming, Montana and Idaho|United States|1872|area=898300|the first national park in the world, established March 1, 1872; Old Faithful and about half the world's geysers
Yosemite National Park;yosemite|national park|Sierra Nevada, California|United States|1890|area=308300|established October 1, 1890; El Capitan, Half Dome and Yosemite Falls
Grand Canyon National Park|national park|Arizona|United States|1919|area=493000|established February 26, 1919
Zion National Park|national park|Utah|United States|1919|area=59900|established November 19, 1919
Great Smoky Mountains National Park|national park|Tennessee and North Carolina|United States|1934|area=211400|the most visited US national park
Rocky Mountain National Park|national park|Colorado|United States|1915|area=107600|established January 26, 1915
Glacier National Park|national park|Montana|United States|1910|area=410000|established May 11, 1910
Denali National Park;denali national park and preserve|national park|Alaska|United States|1917|area=2458500|surrounds Denali, the highest peak in North America
Everglades National Park|national park|Florida|United States|1947|area=610700|the largest subtropical wilderness in the United States
Moab Khotsong|mine|North West Province|South Africa|2003|depth=3054|
";

fn all() -> &'static Vec<Structure> {
    static S: OnceLock<Vec<Structure>> = OnceLock::new();
    S.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                if f.len() != 7 {
                    return None;
                }
                Some(Structure {
                    names: f[0].split(';').map(String::from).collect(),
                    kind: f[1].into(),
                    location: f[2].into(),
                    country: f[3].into(),
                    opened: f[4].into(),
                    nums: f[5].split(';').filter_map(|kv| kv.split_once('=').and_then(|(k, v)| Some((k.to_string(), v.parse().ok()?)))).collect(),
                    note: f[6].into(),
                })
            })
            .collect()
    })
}

fn find(s: &str) -> Option<&'static Structure> {
    let s = s.trim().trim_start_matches("the ").trim().trim_end_matches("'s");
    let s = s.trim_end_matches(" historical site").trim_end_matches(" historic site");
    all().iter().find(|x| x.names.iter().any(|n| n.to_lowercase() == s))
}

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

fn show(k: &str, v: f64) -> String {
    match k {
        "height" | "span" | "drop" | "depth" => format!("{} m ({} ft)", n(v), n((v * 3.280_84).round())),
        "length" if v >= 10_000.0 => format!("{} km ({} mi)", n(v / 1000.0), n((v / 1609.344 * 100.0).round() / 100.0)),
        "length" => format!("{} m ({} ft)", n(v), n((v * 3.280_84).round())),
        "area" => format!("{} ha ({} acres)", n(v), n((v * 2.471_05).round())),
        "seats" => format!("{} seats", n(v)),
        "speed" => format!("{} km/h ({} mph)", n(v), n((v / 1.609_344).round())),
        "power" => format!("{} MW", n(v)),
        "floors" => format!("{} floors", n(v)),
        _ => n(v),
    }
}

fn label(k: &str) -> &str {
    match k {
        "span" => "main span",
        "drop" => "drop",
        "power" => "installed capacity",
        "seats" => "seating capacity",
        other => other,
    }
}

fn props(x: &Structure) -> Content {
    let mut r = vec![vec![text("type"), text(x.kind.clone())], vec![text("location"), text(format!("{}, {}", x.location, x.country))], vec![text("opened"), text(x.opened.clone())]];
    for (k, v) in &x.nums {
        r.push(vec![text(label(k)), text(show(k, *v))]);
    }
    if !x.note.is_empty() {
        r.push(vec![text("note"), text(x.note.clone())]);
    }
    table(None, r)
}

fn card(x: &Structure) -> Answer {
    let lead = x.nums.first().map(|(k, v)| format!("{} {}", label(k), show(k, *v))).unwrap_or_default();
    Answer::new("structure", vec![
        pod("Input", "Input interpretation", text(format!("{} ({})", x.name(), x.kind))),
        primary(pod("Result", "Basic properties", text(format!("{} in {}, {}; opened {}{}", x.kind, x.location, x.country, x.opened, if lead.is_empty() { String::new() } else { format!("; {lead}") })))),
        pod("Properties", "Properties", props(x)),
    ])
}

fn attr_word(w: &str) -> Option<&'static str> {
    Some(match w {
        "height" | "tall" | "how tall" => "height",
        "length" | "long" | "how long" => "length",
        "span" | "main span" => "span",
        "area" | "size" => "area",
        "seating capacity" | "capacity" | "seats" | "number of seats" => "seats",
        "speed" | "top speed" | "how fast" => "speed",
        "power" | "power capacity" | "installed capacity" | "generating capacity" => "power",
        "depth" | "how deep" => "depth",
        "floors" | "number of floors" | "stories" => "floors",
        "opened" | "opening date" | "year opened" | "when was" | "completed" | "built" => "opened",
        "physical properties" | "properties" | "dimensions" => "all",
        _ => return None,
    })
}

fn split(s: &str) -> Option<Vec<&'static Structure>> {
    let parts: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|/|\bvs\.?\b|\bversus\b|\band\b|&)\s*").split(s).filter(|x| !x.trim().is_empty()).collect();
    parts.iter().map(|p| find(p)).collect()
}

fn compare(xs: &[&Structure], attr: &str) -> Answer {
    let val = |x: &Structure| -> String {
        match attr {
            "opened" => x.opened.clone(),
            "all" if x.nums.is_empty() => format!("built {}", x.opened),
            "all" => x.nums.iter().map(|(k, v)| format!("{} {}", label(k), show(k, *v))).collect::<Vec<_>>().join(", "),
            k => x.num(k).map(|v| show(k, v)).unwrap_or_else(|| "(not available)".into()),
        }
    };
    let main = xs.iter().map(|x| format!("{}: {}", x.name(), val(x))).collect::<Vec<_>>().join(" | ");
    let mut keys: Vec<String> = vec![];
    for x in xs {
        for (k, _) in &x.nums {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
    }
    let mut head = vec![""];
    head.extend(xs.iter().map(|x| x.name()));
    let mut t = vec![];
    let mut row = vec![text("opened")];
    row.extend(xs.iter().map(|x| text(x.opened.clone())));
    t.push(row);
    for k in &keys {
        let mut row = vec![text(label(k))];
        row.extend(xs.iter().map(|x| text(x.num(k).map(|v| show(k, v)).unwrap_or_default())));
        t.push(row);
    }
    let title = if attr == "all" { "Properties".to_string() } else { label(attr).to_string() };
    Answer::new("structures", vec![
        pod("Input", "Input interpretation", text(xs.iter().map(|x| x.name()).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", &title, text(main))),
        pod("Table", "Comparison", table(Some(head), t)),
    ])
}

fn country_lists(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:list of |famous |all )?(castles|dams|bridges|tunnels|roller coasters|theatres|theaters|canals|skyscrapers|towers|mines|theme parks) (?:in|of) (?:the )?(.+)$").captures(l)?;
    let kind = match &m[1] {
        "castles" => "castle",
        "dams" => "dam",
        "bridges" => "bridge",
        "tunnels" => "tunnel",
        "roller coasters" => "roller coaster",
        "theatres" | "theaters" => "theat",
        "canals" => "canal",
        "skyscrapers" => "skyscraper",
        "towers" => "tower",
        "mines" => "mine",
        _ => "theme park",
    };
    let place = match m[2].trim() {
        "uk" | "united kingdom" | "britain" | "great britain" => "United Kingdom",
        "us" | "usa" | "united states" | "america" => "United States",
        other => {
            let o = other.to_string();
            return all().iter().find(|x| x.country.to_lowercase() == o).map(|x| x.country.as_str()).and_then(|c| list(kind, c, &m[1]));
        }
    };
    list(kind, place, &m[1])
}

fn list(kind: &str, place: &str, word: &str) -> Option<Answer> {
    let xs: Vec<&Structure> = all().iter().filter(|x| x.kind.contains(kind) && x.country.contains(place)).collect();
    if xs.is_empty() {
        return None;
    }
    let t = xs.iter().map(|x| vec![text(x.name()), text(x.location.clone()), text(x.opened.clone())]).collect();
    Some(Answer::new("structures", vec![
        pod("Input", "Input interpretation", text(format!("{word} in {place}"))),
        primary(pod("Result", "Result", text(xs.iter().map(|x| x.name()).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Notable examples", table(Some(vec!["name", "location", "built"]), t)),
    ]))
}

fn deep_mines(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:mines|deepest mines|mines deeper than|mines over|mines more than) ?(?:(\d+(?:\.\d+)?) ?(feet|ft|meters|m|km))?(?: deep)?$").captures(l)?;
    let min = match (m.get(1), m.get(2)) {
        (Some(v), Some(u)) => {
            let v: f64 = v.as_str().parse().ok()?;
            match u.as_str() {
                "feet" | "ft" => v * 0.3048,
                "km" => v * 1000.0,
                _ => v,
            }
        }
        _ if l.starts_with("deepest") => 0.0,
        _ => return None,
    };
    let mut xs: Vec<&Structure> = all().iter().filter(|x| x.kind == "mine" && x.num("depth").map_or(false, |d| d > min)).collect();
    xs.sort_by(|a, b| b.num("depth").partial_cmp(&a.num("depth")).unwrap());
    let t = xs.iter().map(|x| vec![text(x.name()), text(show("depth", x.num("depth").unwrap())), text(x.country.clone())]).collect();
    Some(Answer::new("structures", vec![
        pod("Input", "Input interpretation", text(format!("mines deeper than {}", show("depth", min)))),
        primary(pod("Result", "Result", text(xs.iter().map(|x| x.name()).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Deepest mines", table(Some(vec!["mine", "depth", "country"]), t)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    if let Some(a) = country_lists(l).or_else(|| deep_mines(l)) {
        return Some(a);
    }
    // "Great Buddha of Thailand / Big Lobster height": the property may come last
    let moved;
    let l = if let Some(m) = re!(r"^(.+(?:/|,| vs\.? ).+?) (height|length|area|depth|top speed|seating capacity)$").captures(l) {
        moved = format!("{} {}", &m[2], &m[1]);
        moved.as_str()
    } else {
        l
    };
    // "length Three Gorges Dam / Hoover Dam", "seating capacity Muny Theater vs Broadway Theater",
    // "area of Tokyo Disneyland, Disneyland", "physical properties of Kingda Ka, Goliath, Furius Baco"
    if let Some(m) = re!(r"^(?:the )?(height|length|main span|span|area|size|seating capacity|capacity|number of seats|seats|top speed|speed|power capacity|installed capacity|generating capacity|power|depth|number of floors|floors|opening date|year opened|physical properties|properties|dimensions)(?: of)? (.+)$").captures(l) {
        let attr = attr_word(&m[1])?;
        let xs = split(&m[2])?;
        if xs.len() == 1 && attr != "opened" && attr != "all" {
            let x = xs[0];
            let v = x.num(attr)?;
            return Some(Answer::new("structure", vec![pod("Input", "Input interpretation", text(format!("{} {}", x.name(), label(attr)))), primary(pod("Result", label(attr), text(show(attr, v)))), pod("Properties", "Properties", props(x))]));
        }
        return Some(compare(&xs, attr));
    }
    // "how tall is the Eiffel Tower", "when was the Brooklyn Bridge built"
    if let Some(m) = re!(r"^(how tall|how long|how deep|how fast) is (.+)$").captures(l) {
        let x = find(&m[2])?;
        let attr = attr_word(&m[1])?;
        let v = x.num(attr)?;
        return Some(Answer::new("structure", vec![pod("Input", "Input interpretation", text(format!("{} {}", x.name(), label(attr)))), primary(pod("Result", label(attr), text(show(attr, v)))), pod("Properties", "Properties", props(x))]));
    }
    if let Some(m) = re!(r"^when (?:was|were) (.+?) (?:built|opened|completed|constructed)$").captures(l).or_else(|| re!(r"^(.+?) (?:opened|opening date|year built|built)$").captures(l)) {
        let xs = split(&m[1])?;
        return Some(compare(&xs, "opened"));
    }
    let xs = split(l)?;
    if xs.len() == 1 {
        // a few names are also ordinary words or places
        if matches!(l, "goliath" | "muny" | "disneyland" | "the met" | "grand canal" | "big ben" | "magic kingdom") {
            return if l == "disneyland" || l == "big ben" { Some(card(xs[0])) } else { None };
        }
        return Some(card(xs[0]));
    }
    Some(compare(&xs, "all"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_parses() {
        assert_eq!(DATA.lines().filter(|l| !l.trim().is_empty()).count(), all().len());
    }
}
